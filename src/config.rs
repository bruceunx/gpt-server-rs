use std::env;

pub struct Config {
    pub openai_url: String,
    pub openai_key: String,
    pub openai_subscription_url: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            openai_url: env::var("OPENAI_URL").expect("OPENAI_URL must be set"),
            openai_key: env::var("OPENAI_KEY").expect("OPENAI_KEY must be set"),
            openai_subscription_url: env::var("OPENAI_SUBSCRIPTION_URL").ok(),
        }
    }
}
