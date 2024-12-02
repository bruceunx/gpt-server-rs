use crate::models::prompt::Prompt;
use futures::stream::{Stream, StreamExt};
use reqwest::{header, Client};
use std::pin::Pin;

pub struct OpenAIService {
    client: Client,
    url: String,
    api_key: String,
    subscription_url: String,
}

impl OpenAIService {
    pub fn new(url: String, api_key: String, subscription_url: String) -> Self {
        let client = Client::builder().build().unwrap();
        Self {
            client,
            url,
            api_key,
            subscription_url,
        }
    }

    pub async fn stream_chat(
        &self,
        prompt: Prompt,
    ) -> Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>>>> {
        let mut messages = prompt.messages.clone();

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

    pub async fn remain_token(&self) -> u64 {
        if self.subscription_url.len() == 0 {
            return 0;
        }
        let res = self.client.get(&self.subscription_url).send().await;

        match res {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    let remain_token = data["remain_quota"].as_u64().unwrap_or(0);
                    return remain_token;
                }
                Err(_) => 0,
            },
            Err(_) => 0,
        }
    }
}
