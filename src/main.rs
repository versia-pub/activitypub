use activitypub_federation::{
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
    http_signatures::generate_actor_keypair,
    traits::Actor,
};
use activitystreams_kinds::public;
use actix_web::{
    get, http::KeepAlive, middleware, post, web, App, Error, HttpResponse, HttpServer,
};
use actix_web_prom::PrometheusMetricsBuilder;
use async_once::AsyncOnce;
use chrono::{DateTime, Utc};
use clap::Parser;
use database::Database;
use entities::post;
use http::{http_get_user, http_post_user_inbox, webfinger};
use lysand::http::{create_activity, fetch_lysand_post, fetch_post, fetch_user, query_post};
use objects::person::{DbUser, Person};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::ToSocketAddrs,
    sync::{Arc, Mutex, OnceLock},
};
use tokio::signal;
use tracing::{info, instrument::WithSubscriber};
use url::Url;
use utils::generate_object_id;
use uuid::Uuid;

use crate::{
    activities::create_post::CreatePost,
    database::{Config, State},
    objects::post::{Mention, Note},
};
use crate::{activities::follow::Follow, entities::user};
use dotenv::dotenv;
use lazy_static::lazy_static;

mod activities;
mod database;
mod entities;
mod error;
mod http;
mod lysand;
mod objects;
mod utils;

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    health: bool,
}

#[derive(Parser, Debug)]
#[clap(author = "April John", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// optional parse arg for config file
    #[arg()]
    config_file: Option<String>,
}

#[get("/")]
async fn index(_: web::Data<State>) -> actix_web::Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(Response { health: true }))
}

#[get("/test/postmanually/{user}/{post}")]
async fn post_manually(
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let local_user = state.local_user().await?;
    let data = FEDERATION_CONFIG.get().unwrap();
    let target =
        webfinger_resolve_actor::<State, user::Model>(path.0.as_str(), &data.to_request_data())
            .await?;

    let mention = Mention {
        href: Url::parse(&target.id)?,
        kind: Default::default(),
    };
    // TODO change
    let uuid = uuid::Uuid::now_v7().to_string();
    let id: ObjectId<post::Model> = generate_object_id(data.domain(), &uuid)?.into();
    let note = Note {
        kind: Default::default(),
        id: id.clone(),
        sensitive: Some(false),
        attributed_to: Url::parse(&local_user.id).unwrap().into(),
        to: vec![public(), mention.href.clone()],
        content: format!("{} {}", path.1, target.name),
        tag: vec![mention],
        in_reply_to: None,
        cc: vec![].into(),
    };

    let post = entities::post::ActiveModel {
        id: Set(uuid),
        creator: Set(local_user.id.clone()),
        content: Set(note.content.clone()),
        sensitive: Set(false),
        created_at: Set(Utc::now()),
        local: Set(true),
        updated_at: Set(Some(Utc::now())),
        content_type: Set("Note".to_string()),
        visibility: Set("public".to_string()),
        url: Set(id.to_string()),
        ap_json: Set(Some(serde_json::to_string(&note).unwrap())),
        ..Default::default()
    };

    let post = post.insert(DB.get().unwrap()).await?;

    CreatePost::send(
        note,
        post,
        target.shared_inbox_or_inbox(),
        &data.to_request_data(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(Response { health: true }))
}

#[get("/test/follow/{user}")]
async fn follow_manually(
    path: web::Path<String>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let local_user = state.local_user().await?;
    let data = FEDERATION_CONFIG.get().unwrap();
    let followee =
        webfinger_resolve_actor::<State, user::Model>(path.as_str(), &data.to_request_data())
            .await?;

    let followee_object: ObjectId<user::Model> = Url::parse(&followee.url)?.into();
    let localuser_object: ObjectId<user::Model> = Url::parse(&local_user.url)?.into();

    Follow::send(
        localuser_object,
        followee_object,
        followee.shared_inbox_or_inbox(),
        &data.to_request_data(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(Response { health: true }))
}

const DOMAIN_DEF: &str = "social.lysand.org";
const LOCAL_USER_NAME: &str = "apservice";

lazy_static! {
    static ref SERVER_URL: String = env::var("LISTEN").unwrap_or("0.0.0.0:8080".to_string());
    static ref DATABASE_URL: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    static ref USERNAME: String =
        env::var("LOCAL_USER_NAME").unwrap_or(LOCAL_USER_NAME.to_string());
    static ref API_DOMAIN: String = env::var("API_DOMAIN").expect("not set API_DOMAIN");
    static ref LYSAND_DOMAIN: String = env::var("LYSAND_DOMAIN").expect("not set LYSAND_DOMAIN");
    static ref FEDERATED_DOMAIN: String =
        env::var("FEDERATED_DOMAIN").unwrap_or(API_DOMAIN.to_string());
}

static DB: OnceLock<DatabaseConnection> = OnceLock::new();
static FEDERATION_CONFIG: OnceLock<FederationConfig<State>> = OnceLock::new();

#[actix_web::main]
async fn main() -> actix_web::Result<(), anyhow::Error> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let uuid = Uuid::now_v7();

    let ap_id = Url::parse(&format!(
        "https://{}/apbridge/user/{}",
        API_DOMAIN.to_string(),
        &uuid.to_string()
    ))?;
    let inbox = Url::parse(&format!(
        "https://{}/{}/inbox",
        API_DOMAIN.to_string(),
        &USERNAME.to_string()
    ))?;
    let keypair = generate_actor_keypair()?;

    let ap_json = Person {
        id: ap_id.clone().into(),
        preferred_username: USERNAME.to_string(),
        name: "Test account <3".to_string(),
        inbox: inbox.clone(),
        public_key: activitypub_federation::protocol::public_key::PublicKey {
            owner: ap_id.clone(),
            public_key_pem: keypair.public_key.clone(),
            id: format!("{}#main-key", ap_id.clone()),
        },
        summary: Some("Test account <3".to_string()),
        url: ap_id.clone(),
        kind: Default::default(),
        indexable: Some(false),
        discoverable: Some(false),
        icon: None,
        image: None,
        attachment: None,
        tag: None,
        endpoints: None,
        followers: None,
        following: None,
        featured: None,
        outbox: None,
        also_known_as: None,
        featured_tags: None,
        manually_approves_followers: Some(false),
    };

    let user = entities::user::ActiveModel {
        id: Set(uuid.to_string()),
        username: Set(USERNAME.to_string()),
        name: Set("Test account <3".to_string()),
        inbox: Set(inbox.to_string()),
        public_key: Set(keypair.public_key.clone()),
        private_key: Set(Some(keypair.private_key.clone())),
        last_refreshed_at: Set(Utc::now()),
        follower_count: Set(0),
        following_count: Set(0),
        url: Set(ap_id.to_string()),
        local: Set(true),
        created_at: Set(Utc::now()),
        ap_json: Set(Some(serde_json::to_string(&ap_json).unwrap())),
        ..Default::default()
    };

    let db = sea_orm::Database::connect(DATABASE_URL.to_string()).await?;

    info!("Connected to database: {:?}", db);

    DB.set(db)
        .expect("We were not able to save the DB conn into memory");

    let db = DB.get().unwrap();

    let user = user.insert(db).await;

    if let Err(err) = user {
        eprintln!("Error inserting user: {:?}", err);
    } else {
        info!("User inserted: {:?}", user.unwrap());
    }

    let state: State = State {
        database_connection: Arc::new(db.clone()),
    };

    let data = FederationConfig::builder()
        .domain(FEDERATED_DOMAIN.to_string())
        .app_data(state.clone())
        .http_signature_compat(true)
        .signed_fetch_actor(&state.local_user().await.unwrap())
        .build()
        .await?;

    let _ = FEDERATION_CONFIG.set(data.clone());

    let mut labels = HashMap::new();
    labels.insert("domain".to_string(), FEDERATED_DOMAIN.to_string());
    labels.insert("name".to_string(), USERNAME.to_string());
    labels.insert("api_domain".to_string(), API_DOMAIN.to_string());

    let prometheus = PrometheusMetricsBuilder::new("activitypub_bridge")
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .unwrap();

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .wrap(prometheus.clone())
            .wrap(FederationMiddleware::new(data.clone()))
            .service(post_manually)
            .service(follow_manually)
            .route("/{user}", web::get().to(http_get_user))
            .route("/{user}/inbox", web::post().to(http_post_user_inbox))
            .route(
                "/apbridge/{user}/inbox",
                web::post().to(http_post_user_inbox),
            )
            .route("/.well-known/webfinger", web::get().to(webfinger))
            .service(index)
            .service(fetch_post)
            .service(fetch_user)
            .service(create_activity)
            .service(query_post)
            .service(fetch_lysand_post)
    })
    .bind(SERVER_URL.to_string())?
    .workers(num_cpus::get())
    .shutdown_timeout(20)
    .keep_alive(KeepAlive::Os)
    .run();

    tokio::spawn(http_server);

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }

    info!("Main thread shutdown..");

    Ok(())
}
