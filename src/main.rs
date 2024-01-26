use actix_web::{get, middleware, web, App, Error, HttpResponse, HttpServer};
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
struct State {
    db: DatabaseConnection,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    health: bool,
}

#[get("/")]
async fn index(_: web::Data<State>) -> actix_web::Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(Response { health: true }))
}

#[actix_web::main]
async fn main() -> actix_web::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server_url = env::var("LISTEN").unwrap_or("127.0.0.1:8080".to_string());

    let mut opts =
        sea_orm::ConnectOptions::new(env::var("DATABASE_URL").expect("DATABASE_URL ust be set"));
    opts.max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));

    let db: DatabaseConnection = Database::connect(opts)
        .await
        .expect("Failed to connect to database");

    let state = State { db };

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .service(index)
    })
    .bind(&server_url)?
    .run()
    .await?;

    Ok(())
}
