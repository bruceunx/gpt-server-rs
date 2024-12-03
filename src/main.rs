use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod handlers;
mod models;
mod routes;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    HttpServer::new(|| {
        App::new().service(
            web::scope("/v1")
                .service(routes::openai::openai_routes())
                .service(routes::claude::claude_routes()),
        )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
