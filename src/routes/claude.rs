use crate::handlers;
use actix_web::{web, Scope};

use crate::services::{ApiServiceManager, ApiSupplier};
use std::env;

pub fn claude_routes() -> Scope {
    let openai_supplier = ApiSupplier::Claude {
        url: env::var("CLAUDE_URL").unwrap_or("".to_string()),
        api_key: env::var("CLAUDE_KEY").unwrap_or("".to_string()),
        model: env::var("CLAUDE_MODEL").unwrap_or("gpt-4o-turbo".to_string()),
    };

    let openai_service = web::Data::new(ApiServiceManager::new(openai_supplier));

    web::scope("/claude")
        .app_data(openai_service.clone())
        .route("/chat", web::post().to(handlers::chat_handler))
}
