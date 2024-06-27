use activitypub_federation::{
    protocol::context::WithContext, traits::Object, FEDERATION_CONTENT_TYPE,
};
use activitystreams_kinds::{activity::CreateType, object};
use actix_web::{get, web, HttpResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    database::State,
    entities::{
        post::{self, Entity},
        prelude,
    },
    error, objects,
    utils::{base_url_decode, generate_create_id},
    Response, DB, FEDERATION_CONFIG,
};

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
        .json(
            crate::objects::post::Note::from_db(&post)
        ))
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
