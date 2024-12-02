use crate::models::prompt::Prompt;
use futures::stream::{Stream, StreamExt};
use reqwest::{header, Client};
use std::pin::Pin;

pub struct OpenAIService {
    client: Client,
    url: String,
    api_key: String,
}

impl OpenAIService {
    pub fn new(url: String, api_key: String) -> Self {
        let client = Client::new();
        Self {
            client,
            url,
            api_key,
        }
    }

    pub async fn stream_chat(
        &self,
        prompt: Prompt,
    ) -> Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>>>> {
        let mut messages = prompt.messages.clone();

        // Limit messages if too many
        if messages.len() > 6 {
            messages = messages.into_iter().rev().take(4).collect();
        }

        let request_body = serde_json::json!({
            "model": "gpt-4-turbo",
            "stream": true,
            "messages": messages
        });

        let response = self
            .client
            .post(&self.url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await
            .expect("Failed to send request");

        let stream = response
            .bytes_stream()
            .map(|item| item.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()));

        Box::pin(stream)
    }
}
