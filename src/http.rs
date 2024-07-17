use crate::{
    database::StateHandle,
    entities::user,
    error::Error,
    lysand::{
        self,
        conversion::{db_user_from_url, local_db_user_from_name, receive_lysand_note},
    },
    objects::person::{DbUser, PersonAcceptedActivities},
    utils::generate_user_id,
    API_DOMAIN, LYSAND_DOMAIN,
};
use activitypub_federation::{
    actix_web::{inbox::receive_activity, signing_actor},
    config::{Data, FederationConfig, FederationMiddleware},
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name},
    protocol::context::WithContext,
    traits::{Actor, Object},
    FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::anyhow;
use serde::Deserialize;
use tracing::info;
use url::Url;
use webfinger::resolve;

pub fn listen(config: &FederationConfig<StateHandle>) -> Result<(), Error> {
    let hostname = config.domain();
    info!("Listening with actix-web on {hostname}");
    let config = config.clone();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(FederationMiddleware::new(config.clone()))
            //.route("/", web::get().to(http_get_system_user))
            .route("/{user}", web::get().to(http_get_user))
            .route("/{user}/inbox", web::post().to(http_post_user_inbox))
            .route("/.well-known/webfinger", web::get().to(webfinger))
    })
    .bind(hostname)?
    .run();
    tokio::spawn(server);
    Ok(())
}

pub fn lysand_inbox(
    note: web::Json<lysand::objects::Note>,
    id: web::Path<String>,
    data: Data<StateHandle>,
) -> Result<HttpResponse, Error> {
    tokio::spawn(receive_lysand_note(note.into_inner(), id.into_inner()));
    Ok(HttpResponse::Created().finish())
}

/// Handles requests to fetch system user json over HTTP
/*pub async fn http_get_system_user(data: Data<DatabaseHandle>) -> Result<HttpResponse, Error> {
    let json_user = data.system_user.clone().into_json(&data).await?;
    Ok(HttpResponse::Ok()
        .content_type(FEDERATION_CONTENT_TYPE)
        .json(WithContext::new_default(json_user)))
}*/

/// Handles requests to fetch user json over HTTP
pub async fn http_get_user(
    request: HttpRequest,
    user_name: web::Path<String>,
    data: Data<StateHandle>,
) -> Result<HttpResponse, Error> {
    //let signed_by = signing_actor::<DbUser>(&request, None, &data).await?;
    // here, checks can be made on the actor or the domain to which
    // it belongs, to verify whether it is allowed to access this resource
    //info!(
    //    "Fetch user request is signed by system account {}",
    //    signed_by.id()
    //);

    let db_user = data.local_user().await?;
    if user_name.into_inner() == db_user.username {
        let json_user = db_user.into_json(&data).await?;
        Ok(HttpResponse::Ok()
            .content_type(FEDERATION_CONTENT_TYPE)
            .json(WithContext::new_default(json_user)))
    } else {
        Err(anyhow!("Invalid user").into())
    }
}

/// Handles messages received in user inbox
pub async fn http_post_user_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<StateHandle>,
) -> Result<HttpResponse, Error> {
    receive_activity::<WithContext<PersonAcceptedActivities>, user::Model, StateHandle>(
        request, body, &data,
    )
    .await
}

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

pub async fn webfinger(
    query: web::Query<WebfingerQuery>,
    data: Data<StateHandle>,
) -> Result<HttpResponse, Error> {
    let name = extract_webfinger_name(&query.resource, &data)?;
    let user = local_db_user_from_name(name.to_string()).await?;
    Ok(HttpResponse::Ok().json(build_webfinger_response(
        query.resource.clone(),
        generate_user_id(&API_DOMAIN, &user.id)?,
    )))
}
