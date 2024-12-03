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

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    let redis_settings = services::RedisSettings::from_env();

    let redis_client =
        redis_async::client::paired_connect(&redis_settings.redis_host, redis_settings.redis_port)
            .await
            .unwrap();

    HttpServer::new(move || {
        App::new().service(
            web::scope("/v1")
                .service(routes::openai::openai_routes())
                .service(routes::claude::claude_routes())
                .service(routes::gemini::gemini_routes(redis_client.clone())),
        )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
