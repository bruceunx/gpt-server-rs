use super::models::prompt::Prompt;
use chrono::prelude::*;
use futures::stream::{Stream, StreamExt};
use redis_async::{client::PairedConnection, resp::RespValue, resp_array};
use reqwest::{header, Client, Proxy};
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
}

impl RedisSettings {
    pub fn from_env() -> Self {
        RedisSettings {
            redis_host: env::var("REDIS_HOST").unwrap_or("".to_string()),
            redis_port: env::var("REDIS_PORT")
                .map_or_else(|_| 0, |value| value.parse().unwrap_or(0)),
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
        redis_password: String,
    },
}

async fn rate_limit(redis_client: &PairedConnection, rate_limit: u16, password: &str) -> bool {
    match redis_client.send(resp_array!["AUTH", password]).await {
        Ok(()) => {
            let now = Utc::now();
            let current_minute = now.format("%Y-%m-%dT%H:%M").to_string();
            let redis_key = format!("rate_limit_{}", current_minute);

            let current_count: i64 = match redis_client.send(resp_array!["INCR", &redis_key]).await
            {
                Ok(value) => value,
                Err(_) => return false,
            };

            if current_count == 1 {
                redis_client.send_and_forget(resp_array![
                    "EXPIREAT",
                    &redis_key,
                    RespValue::Integer((now + chrono::Duration::minutes(1)).timestamp()),
                ]);
            }

            current_count <= rate_limit as i64
        }
        Err(_) => false,
    }
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
                redis_password,
                ..
            } => {
                if rate_limit(redis_client, *rate_limit_per_minute, redis_password).await {
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
                redis_password,
                ..
            } => {
                if rate_limit(redis_client, *rate_limit_per_minute, redis_password).await {
                    pro_model
                } else {
                    model
                }
            }
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

        let response = match self.supplier {
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

                let request_body = serde_json::json!({
                    "model": self.supplier.get_model().await,
                    "contents": contents,
                });

                self.client
                    .post(self.supplier.get_url().await)
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to send request: {}", e))?
            }
            _ => {
                let request_body = serde_json::json!({
                    "model": self.supplier.get_model().await,
                    "stream": true,
                    "messages": truncated_messages
                });

                self.client
                    .post(self.supplier.get_url().await)
                    .header(
                        header::AUTHORIZATION,
                        format!("Bearer {}", self.supplier.get_api_key()),
                    )
                    .header(header::CONTENT_TYPE, "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to send request: {}", e))?
            }
        };

        let stream = response
            .bytes_stream()
            .map(|item| item.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()));

        Ok(Box::pin(stream))
    }
}
