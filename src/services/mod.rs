use super::models::prompt::Prompt;
use chrono::prelude::*;
use futures::stream::{Stream, StreamExt};
use redis_async::{client::PairedConnection, resp_array};
use reqwest::{header, Client, Proxy};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::pin::Pin;

pub struct ApiServiceManager {
    client: Client,
    supplier: ApiSupplier,
}

pub struct RedisSettings {
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: String,
}

impl RedisSettings {
    pub fn from_env() -> Self {
        RedisSettings {
            redis_host: env::var("REDIS_HOST").unwrap_or("".to_string()),
            redis_port: env::var("REDIS_PORT")
                .map_or_else(|_| 0, |value| value.parse().unwrap_or(0)),
            redis_password: env::var("REDIS_PASSWORD").unwrap_or("".to_string()),
        }
    }
}

#[derive(Clone)]
pub enum ApiSupplier {
    OpenAi {
        url: String,
        model: String,
        api_key: String,
    },
    Claude {
        url: String,
        model: String,
        api_key: String,
    },
    Gemini {
        redis_client: PairedConnection,
        url: String,
        model: String,
        api_key: String,
        pro_url: String,
        pro_model: String,
        rate_limit_per_minute: u16,
    },
}

async fn rate_limit(redis_client: &PairedConnection, rate_limit: u16) -> bool {
    let connect_inner = redis_client.clone();
    let now = Utc::now();
    let current_minute = now.format("%Y-%m-%dT%H:%M").to_string();
    let redis_key = format!("rate_limit_{}", current_minute);

    let current_count: i64 = match connect_inner.send(resp_array!["INCR", &redis_key]).await {
        Ok(value) => value,
        Err(_) => return false,
    };

    if current_count == 1 {
        let expire_timestamp = (now + chrono::Duration::minutes(1)).timestamp();
        connect_inner.send_and_forget(resp_array![
            "EXPIREAT",
            &redis_key,
            expire_timestamp.to_string()
        ]);
    }

    current_count <= rate_limit as i64
}

impl ApiSupplier {
    pub async fn get_url(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { url, .. } => url,
            ApiSupplier::Claude { url, .. } => url,
            ApiSupplier::Gemini {
                url,
                redis_client,
                rate_limit_per_minute,
                pro_url,
                ..
            } => {
                if rate_limit(redis_client, *rate_limit_per_minute).await {
                    pro_url
                } else {
                    url
                }
            }
        }
    }

    pub async fn get_model(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { model, .. } => model,
            ApiSupplier::Claude { model, .. } => model,
            ApiSupplier::Gemini {
                model,
                pro_model,
                redis_client,
                rate_limit_per_minute,
                ..
            } => {
                if rate_limit(redis_client, *rate_limit_per_minute).await {
                    pro_model
                } else {
                    model
                }
            }
        }
    }

    pub async fn get_gemini_model_url(&self) -> (&str, &str) {
        match self {
            ApiSupplier::Gemini {
                redis_client,
                url,
                model,
                pro_url,
                pro_model,
                rate_limit_per_minute,
                ..
            } => {
                if rate_limit(redis_client, *rate_limit_per_minute).await {
                    (pro_model, pro_url)
                } else {
                    (model, url)
                }
            }
            _ => panic!("not support other ai suppliers"),
        }
    }

    pub fn get_api_key(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { api_key, .. } => api_key,
            ApiSupplier::Claude { api_key, .. } => api_key,
            ApiSupplier::Gemini { api_key, .. } => api_key,
        }
    }
}

impl ApiServiceManager {
    pub fn new(supplier: ApiSupplier) -> Self {
        let client = Client::builder().build().unwrap();
        Self { client, supplier }
    }

    pub async fn chat_stream(
        &self,
        prompt: Prompt,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>>>>, String> {
        let mut truncated_messages = prompt.messages;

        if truncated_messages.len() > 6 {
            truncated_messages = truncated_messages
                .iter()
                .enumerate()
                .filter(|(index, _)| *index == 0 || *index >= truncated_messages.len() - 3)
                .map(|(_, message)| message.clone())
                .collect();
        }

        match self.supplier {
            ApiSupplier::Gemini { .. } => {
                let mut contents: Vec<Value> = Vec::new();
                let mut sys_message: Option<String> = None;

                for message in &truncated_messages {
                    let mut single_message: HashMap<&str, Value> = HashMap::new();

                    if message.role == "system" {
                        sys_message = Some(message.content.replace("GitHub Copilot", "Gemini"));
                        continue;
                    } else if message.role == "assistant" {
                        single_message.insert("role", json!("model"));
                        single_message.insert("parts", json!([{ "text": message.content }]));
                    } else {
                        single_message.insert("role", json!(message.role));
                        let text = if let Some(ref sys_msg) = sys_message {
                            let combined = format!("{} {}", sys_msg, message.content);
                            sys_message = None;
                            combined
                        } else {
                            message.content.clone()
                        };
                        single_message.insert("parts", json!([{ "text": text }]));
                    }

                    contents.push(json!(single_message));
                }

                let (model, url) = self.supplier.get_gemini_model_url().await;

                let request_body = serde_json::json!({
                    "model": model,
                    "contents": contents,
                });
                let response = self
                    .client
                    .post(url)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to send request: {}", e))?;

                let stream = response.bytes_stream().map(|item| {
                    item.map(|bytes| {
                        let chunk = String::from_utf8_lossy(&bytes);

                        if chunk.trim().is_empty() {
                            return "data: [DONE]".to_owned();
                        }

                        match parse_and_transform_chunk(&chunk) {
                            Ok(openai_response) => openai_response.to_owned(),
                            Err(_) => "data: [DONE]".to_owned(),
                        }
                    })
                });

                let full_stream = stream.chain(futures::stream::once(async {
                    Ok("data: [DONE]".to_owned())
                }));

                Ok(Box::pin(full_stream))
            }
            _ => {
                let request_body = serde_json::json!({
                    "model": self.supplier.get_model().await,
                    "stream": true,
                    "messages": truncated_messages
                });

                let response = self
                    .client
                    .post(self.supplier.get_url().await)
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", self.supplier.get_api_key()),
                    )
                    .header(header::CONTENT_TYPE, "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to send request: {}", e))?;

                let stream = response
                    .bytes_stream()
                    .map(|item| item.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()));
                Ok(Box::pin(stream))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GeminiCandidate {
    content: GeminiContent,
    // finishReason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Serialize)]
struct OpenAIChoice {
    index: usize,
    delta: OpenAIDelta,
}

#[derive(Debug, Serialize)]
struct OpenAIDelta {
    content: String,
}

fn parse_and_transform_chunk(chunk: &str) -> Result<String, String> {
    let cleaned_chunk = chunk.trim_start_matches("data: ").trim();
    if cleaned_chunk.is_empty() {
        return Err("Empty chunk".to_string());
    }
    let gemini_response: GeminiResponse =
        serde_json::from_str(cleaned_chunk).map_err(|e| format!("JSON parse error: {}", e))?;

    if let Some(candidate) = gemini_response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            let openai_response = OpenAIResponse {
                choices: vec![OpenAIChoice {
                    index: 0,
                    delta: OpenAIDelta {
                        content: part.text.clone(),
                    },
                }],
            };

            serde_json::to_string(&openai_response)
                .map(|json_str| format!("data: {}\n\n", json_str))
                .map_err(|e| format!("Serialization error: {}", e))
        } else {
            Err("No parts found in Gemini response".to_string())
        }
    } else {
        Err("No candidates found in Gemini response".to_string())
    }
}
