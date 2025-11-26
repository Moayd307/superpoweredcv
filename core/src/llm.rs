use crate::config::LlmConfig;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub struct LlmClient {
    config: LlmConfig,
    client: reqwest::blocking::Client,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn generate(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
        };

        let url = format!("{}/chat/completions", self.config.api_base_url.trim_end_matches('/'));
        
        let mut builder = self.client.post(&url)
            .json(&request);

        if let Some(key) = &self.config.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = builder.send()?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        let response_body: ChatCompletionResponse = response.json()?;
        
        if let Some(choice) = response_body.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from LLM".into())
        }
    }
}
