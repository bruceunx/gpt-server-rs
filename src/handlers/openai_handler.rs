use crate::models::prompt::Prompt;
use crate::services::openai_service::OpenAIService;
use actix_web::{web, HttpResponse, Responder};
use futures::stream::TryStreamExt;

pub async fn chat_handler(
    prompt: web::Json<Prompt>,
    openai_service: web::Data<OpenAIService>,
) -> impl Responder {
    let stream = openai_service.stream_chat(prompt.into_inner()).await;

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(
            stream
                .map_ok(|text| actix_web::web::Bytes::from(text))
                .map_err(|_| actix_web::error::ErrorInternalServerError("Stream error")),
        )
}

pub async fn state_handler(openai_service: web::Data<OpenAIService>) -> impl Responder {
    let remain_token = openai_service.remain_token().await;
    HttpResponse::Ok().json(serde_json::json!({
        "remain_token": remain_token
    }))
}
