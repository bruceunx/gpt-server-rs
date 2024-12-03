use crate::models::prompt::Prompt;
use crate::services::ApiServiceManager;
use actix_web::{web, HttpResponse, Responder};
use futures::stream::TryStreamExt;

pub async fn chat_handler(
    prompt: web::Json<Prompt>,
    openai_service: web::Data<ApiServiceManager>,
) -> impl Responder {
    let stream = openai_service
        .chat_stream(prompt.into_inner())
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(
            stream
                .map_ok(|text| actix_web::web::Bytes::from(text))
                .map_err(|_| actix_web::error::ErrorInternalServerError("Stream error")),
        )
}
