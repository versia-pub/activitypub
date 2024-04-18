use activitypub_federation::{
    config::{FederationConfig, FederationMiddleware},
    http_signatures::generate_actor_keypair,
};
use actix_web::{get, http::KeepAlive, middleware, web, App, Error, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use chrono::{DateTime, Utc};
use clap::Parser;
use database::Database;
use http::{http_get_user, http_post_user_inbox, webfinger};
use objects::person::DbUser;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
};
use tokio::signal;
use tracing::info;
use url::Url;

use crate::database::{Config, State};

mod activities;
mod database;
mod entities;
mod error;
mod http;
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

const DOMAIN: &str = "example.com";
const LOCAL_USER_NAME: &str = "example";

#[actix_web::main]
async fn main() -> actix_web::Result<(), anyhow::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server_url = env::var("LISTEN").unwrap_or("127.0.0.1:8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let local_user = DbUser::new(
        env::var("FEDERATED_DOMAIN")
            .unwrap_or(DOMAIN.to_string())
            .as_str(),
        env::var("LOCAL_USER_NAME")
            .unwrap_or(LOCAL_USER_NAME.to_string())
            .as_str(),
    )
    .unwrap();

    let username = env::var("LOCAL_USER_NAME").unwrap_or(LOCAL_USER_NAME.to_string());
    let domain = env::var("FEDERATED_DOMAIN").unwrap_or(DOMAIN.to_string());

    let ap_id = Url::parse(&format!("https://{}/{}", domain, &username))?;
    let inbox = Url::parse(&format!("https://{}/{}/inbox", domain, &username))?;
    let keypair = generate_actor_keypair()?;

    let user = entities::user::ActiveModel {
        id: Set(ap_id.into()),
        username: Set(username),
        inbox: Set(inbox.to_string()),
        public_key: Set(keypair.public_key.clone()),
        private_key: Set(Some(keypair.private_key.clone())),
        last_refreshed_at: Set(chrono::offset::Utc::now()),
        local: Set(true),
        ..Default::default()
    };

    let db = sea_orm::Database::connect(database_url).await?;

    let user = user.insert(&db).await;

    let config = Config {};

    let state: State = State {
        database_connection: db.into(),
    };

    let data = FederationConfig::builder()
        .domain(env::var("FEDERATED_DOMAIN").expect("FEDERATED_DOMAIN must be set"))
        .app_data(state.clone())
        .build()
        .await?;

    let mut labels = HashMap::new();
    labels.insert(
        "domain".to_string(),
        env::var("FEDERATED_DOMAIN")
            .expect("FEDERATED_DOMAIN must be set")
            .to_string(),
    );
    labels.insert(
        "name".to_string(),
        env::var("LOCAL_USER_NAME")
            .expect("LOCAL_USER_NAME must be set")
            .to_string(),
    );

    let prometheus = PrometheusMetricsBuilder::new("api")
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
            .route("/{user}", web::get().to(http_get_user))
            .route("/{user}/inbox", web::post().to(http_post_user_inbox))
            .route("/.well-known/webfinger", web::get().to(webfinger))
            .service(index)
    })
    .bind(&server_url)?
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
