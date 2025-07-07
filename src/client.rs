use anyhow::{Context, Result};
use reqwest::Client;
use crate::api::{ApiRequest, ApiResponse, ErrorResponse, Message};
use serde_json;

#[derive(Clone)]
pub struct ConversationClient {
    pub client: Client,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub messages: Vec<Message>,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
}

impl ConversationClient {
    pub fn new(api_key: String, model: String, max_tokens: u32, temperature: f32) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            max_tokens,
            temperature,
            messages: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
        }
    }

    pub async fn send_message(&mut self, user_input: &str) -> Result<String> {
        self.messages.push(Message {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        let request = ApiRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            messages: self.messages.clone(),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to API")?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            let error_response: ErrorResponse = serde_json::from_str(&response_text)
                .context("Failed to parse error response")?;
            anyhow::bail!(
                "API Error ({}): {}",
                error_response.error.error_type,
                error_response.error.message
            );
        }

        let api_response: ApiResponse = serde_json::from_str(&response_text)
            .context("Failed to parse API response")?;

        // Track tokens
        self.total_input_tokens += api_response.usage.input_tokens;
        self.total_output_tokens += api_response.usage.output_tokens;

        let assistant_response = api_response
            .content
            .iter()
            .filter(|block| block.content_type == "text")
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        self.messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_response.clone(),
        });

        Ok(assistant_response)
    }

    pub fn total_tokens(&self) -> u32 {
        self.total_input_tokens + self.total_output_tokens
    }

    pub fn clear_conversation(&mut self) {
        self.messages.clear();
        self.total_input_tokens = 0;
        self.total_output_tokens = 0;
    }
}