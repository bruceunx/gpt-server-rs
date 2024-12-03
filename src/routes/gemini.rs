use crate::handlers;
use actix_web::{web, Scope};

use crate::services::{ApiServiceManager, ApiSupplier};
use redis_async::client::PairedConnection;
use std::env;

pub fn gemini_routes(redis_client: Option<PairedConnection>) -> Scope {
    let gemini_supplier = ApiSupplier::Gemini {
        redis_client: redis_client.clone(),
        url: env::var("GEMINI_URL").unwrap_or("".to_string()),
        pro_url: env::var("GEMINI_PRO_URL").unwrap_or("".to_string()),
        api_key: env::var("GEMINI_KEY").unwrap_or("".to_string()),
        model: env::var("GEMINI_MODEL").unwrap_or("claude-3.5-sonnet".to_string()),
        pro_model: env::var("GEMINI_PRO_MODEL").unwrap_or("claude-3.5-sonnet".to_string()),
        rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
            .map_or_else(|_| 3, |value| value.parse().unwrap_or(3)),
    };

    let openai_service = web::Data::new(ApiServiceManager::new(gemini_supplier));

    web::scope("/gemini")
        .app_data(openai_service.clone())
        .route("/chat", web::post().to(handlers::chat_handler))
}
