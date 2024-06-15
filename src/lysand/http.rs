use std::os::linux::raw::stat;

use activitypub_federation::{traits::Object, FEDERATION_CONTENT_TYPE};
use actix_web::{get, web, HttpResponse};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{database::State, entities::{post::{self, Entity}, prelude}, error, Response, DB, FEDERATION_CONFIG};

#[get("/apbridge/object/{post}")]
async fn post_manually(
    path: web::Path<(String, String)>,
    state: web::Data<State>,
) -> actix_web::Result<HttpResponse, error::Error> {
    let db = DB.get().unwrap();

    let post = prelude::Post::find()
        .filter(post::Column::Id.eq(path.0.as_str()))
        .one(db)
        .await?;

    let post = post.unwrap();

    Ok(HttpResponse::Ok().content_type(FEDERATION_CONTENT_TYPE).json(post.into_json(&FEDERATION_CONFIG.get().unwrap().to_request_data()).await?))
}