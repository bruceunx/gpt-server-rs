use super::models::prompt::Prompt;
use futures::stream::{Stream, StreamExt};
use reqwest::{header, Client};
use std::pin::Pin;
use std::time::Duration;

pub struct ApiServiceManager {
    client: Client,
    supplier: ApiSupplier,
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
}

impl ApiSupplier {
    pub fn get_url(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { url, .. } => url,
            ApiSupplier::Claude { url, .. } => url,
        }
    }

    pub fn get_model(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { model, .. } => model,
            ApiSupplier::Claude { model, .. } => model,
        }
    }

    pub fn get_api_key(&self) -> &str {
        match self {
            ApiSupplier::OpenAi { api_key, .. } => api_key,
            ApiSupplier::Claude { api_key, .. } => api_key,
        }
    }
}

impl ApiServiceManager {
    pub fn new(supplier: ApiSupplier) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
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

        let request_body = serde_json::json!({
            "model": self.supplier.get_model(),
            "stream": true,
            "messages": truncated_messages
        });

        let response = self
            .client
            .post(self.supplier.get_url())
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
