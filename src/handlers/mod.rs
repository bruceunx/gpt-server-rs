use crate::models::prompt::Prompt;
use crate::services::ApiServiceManager;
use actix_web::{web, HttpResponse, Responder};
use futures::stream::TryStreamExt;
use log::error;
use serde_json::json;

pub async fn chat_handler(
    prompt: web::Json<Prompt>,
    openai_service: web::Data<ApiServiceManager>,
) -> impl Responder {
    let res_stream = openai_service.chat_stream(prompt.into_inner()).await;

    match res_stream {
        Ok(stream) => HttpResponse::Ok()
            .content_type("text/event-stream")
            .streaming(
                stream
                    .map_ok(|text| actix_web::web::Bytes::from(text))
                    .map_err(|e| {
                        error!("Stream processing error: {:?}", e);
                        actix_web::error::ErrorInternalServerError("Failed to process stream")
                    }),
            ),
        Err(service_err) => {
            error!("API service error: {:?}", service_err);

            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to initiate chat stream",
                "details": service_err.to_string()
            }))
        }
    }
}
