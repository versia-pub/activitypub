use activitypub_federation::{
    fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
    protocol::{context::WithContext, public_key::PublicKey},
    traits::Object,
    FEDERATION_CONTENT_TYPE,
};
use activitystreams_kinds::{activity::CreateType, object};
use actix_web::{get, post, web, HttpResponse};
use sea_orm::{query, ColumnTrait, EntityTrait, QueryFilter};
use url::Url;

use crate::{
    database::State,
    entities::{
        post::{self, Entity},
        prelude, user,
    },
    error,
    lysand::{
        conversion::{lysand_post_from_db, lysand_user_from_db},
        inbox::inbox_entry,
    },
    objects::{self, person::Person},
    utils::{base_url_decode, generate_create_id, generate_user_id},
    Response, API_DOMAIN, DB, FEDERATION_CONFIG,
};

use super::conversion::db_user_from_url;

#[derive(serde::Deserialize)]
struct LysandQuery {
    // Post url
    url: Option<Url>,
    // User handle
    user: Option<String>,
    // User URL
    user_url: Option<Url>,
}

#[get("/apbridge/lysand/query")]
async fn query_post(
    query: web::Query<LysandQuery>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    if query.url.is_none() && query.user.is_none() && query.user_url.is_none() {
        return Ok(
            HttpResponse::BadRequest().body("Bad Request. Error code: mrrrmrrrmrrawwawwawwa")
        );
    }

    let db = DB.get().unwrap();
    let data = FEDERATION_CONFIG.get().unwrap();

    if let Some(user) = query.user.clone() {
        let target =
            webfinger_resolve_actor::<State, user::Model>(user.as_str(), &data.to_request_data())
                .await?;
        let lysand_user = lysand_user_from_db(target).await?;

        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(lysand_user));
    }

    if let Some(user) = query.user_url.clone() {
        let lysand_user = lysand_url_to_user(user).await?;

        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .json(lysand_user));
    }

    let opt_model = prelude::Post::find()
        .filter(post::Column::Url.eq(query.url.clone().unwrap().as_str()))
        .one(db)
        .await?;
    let target;
    if let Some(model) = opt_model {
        target = model;
    } else {
        target = ObjectId::<post::Model>::from(Url::parse(query.url.clone().unwrap().as_str())?)
            .dereference(&data.to_request_data())
            .await?;
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(lysand_post_from_db(target).await?))
}

#[post("/apbridge/lysand/inbox")]
async fn lysand_inbox(
    body: web::Bytes,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let string = String::from_utf8(body.to_vec())?;
    inbox_entry(&string).await?;
    Ok(HttpResponse::Created().finish())
}

#[get("/apbridge/object/{post}")]
async fn fetch_post(
    path: web::Path<String>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let db = DB.get().unwrap();

    let post = prelude::Post::find()
        .filter(post::Column::Id.eq(path.as_str()))
        .one(db)
        .await?;

    let post = match post {
        Some(post) => post,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    Ok(HttpResponse::Ok()
        .content_type(FEDERATION_CONTENT_TYPE)
        .json(crate::objects::post::Note::from_db(&post)))
}

#[get("/apbridge/user/{user}")]
async fn fetch_user(
    path: web::Path<String>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let db = DB.get().unwrap();

    let user = prelude::User::find()
        .filter(user::Column::Id.eq(path.as_str()))
        .one(db)
        .await?;

    let user = match user {
        Some(user) => user,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    let deserialized_user: Person = serde_json::from_str(user.ap_json.as_ref().unwrap().as_str())?;

    Ok(HttpResponse::Ok()
        .content_type(FEDERATION_CONTENT_TYPE)
        .json(WithContext::new_default(deserialized_user)))
}

#[get("/apbridge/lysand/object/{post}")]
async fn fetch_lysand_post(
    path: web::Path<String>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let db = DB.get().unwrap();

    let post = prelude::Post::find()
        .filter(post::Column::Id.eq(path.as_str()))
        .one(db)
        .await?;

    let post = match post {
        Some(post) => post,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(lysand_post_from_db(post).await?))
}

#[get("/apbridge/create/{id}/{base64url}")]
async fn create_activity(
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let db = DB.get().unwrap();

    let url = base_url_decode(path.1.as_str());

    let post = prelude::Post::find()
        .filter(post::Column::Id.eq(path.0.as_str()))
        .one(db)
        .await?;

    let post = match post {
        Some(post) => post,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    let ap_post = crate::objects::post::Note::from_db(&post);

    let data = FEDERATION_CONFIG.get().unwrap();

    let create = crate::activities::create_post::CreatePost {
        actor: ap_post.attributed_to.clone(),
        to: ap_post.to.clone(),
        object: ap_post,
        kind: CreateType::Create,
        id: generate_create_id(&data.to_request_data().domain(), &path.0, &path.1)?,
    };
    let create_with_context = WithContext::new_default(create);

    Ok(HttpResponse::Ok()
        .content_type(FEDERATION_CONTENT_TYPE)
        .json(create_with_context))
}

pub async fn lysand_url_to_user(url: Url) -> anyhow::Result<super::objects::User> {
    let db = DB.get().unwrap();
    let data = FEDERATION_CONFIG.get().unwrap();

    let opt_model = prelude::User::find()
        .filter(user::Column::Url.eq(url.as_str()))
        .one(db)
        .await?;
    let target;
    if let Some(model) = opt_model {
        target = model;
    } else {
        target = ObjectId::<user::Model>::from(url)
            .dereference(&data.to_request_data())
            .await
            .unwrap();
    }

    Ok(lysand_user_from_db(target).await?)
}

pub async fn lysand_url_to_user_and_model(
    url: Url,
) -> anyhow::Result<(super::objects::User, user::Model)> {
    let db = DB.get().unwrap();
    let data = FEDERATION_CONFIG.get().unwrap();

    let opt_model = prelude::User::find()
        .filter(user::Column::Url.eq(url.as_str()))
        .one(db)
        .await?;
    let target;
    if let Some(model) = opt_model {
        target = model;
    } else {
        target = ObjectId::<user::Model>::from(url)
            .dereference(&data.to_request_data())
            .await
            .unwrap();
    }

    Ok((lysand_user_from_db(target.clone()).await?, target))
}

pub async fn main_lysand_url_to_user_and_model(
    url: Url,
) -> anyhow::Result<(super::objects::User, user::Model)> {
    let db = DB.get().unwrap();
    let data = FEDERATION_CONFIG.get().unwrap();

    let opt_model = prelude::User::find()
        .filter(user::Column::Url.eq(url.as_str()))
        .one(db)
        .await?;
    let target;
    if let Some(model) = opt_model {
        target = model;
    } else {
        target = db_user_from_url(url.clone()).await?;
    }

    Ok((lysand_user_from_db(target.clone()).await?, target))
}
