use crate::config::Config;
use crate::handlers::openai_handler;
use crate::services::openai_service::OpenAIService;
use actix_web::{web, Scope};

pub fn openai_routes() -> Scope {
    let config = Config::new();
    let openai_service = web::Data::new(OpenAIService::new(
        config.openai_url,
        config.openai_key,
        config.openai_subscription_url.unwrap_or("".to_string()),
    ));

    web::scope("/openai")
        .app_data(openai_service.clone())
        .route("/chat", web::post().to(openai_handler::chat_handler))
        .route("/state", web::post().to(openai_handler::state_handler))
}
