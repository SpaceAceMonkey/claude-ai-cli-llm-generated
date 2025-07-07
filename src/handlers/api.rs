// src/handlers/api.rs
use anyhow::{Context, Result};
use tokio::time::Duration;
use crate::api::{ApiRequest, ApiResponse, ErrorResponse, Message};
use reqwest::Client;

pub async fn send_message_to_api(
    user_input: String,
    messages: Vec<Message>,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    simulate_mode: bool,
) -> Result<(String, u32, u32, Vec<Message>)> {
    if simulate_mode {
        // Simulate API delay
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Generate mock response
        let mock_response = format!(
            "This is a simulated response to your message: \"{}\". \
            In simulate mode, no actual API calls are made. \
            Your message had {} characters.",
            user_input.trim(),
            user_input.len()
        );
        
        // Don't add user message - it's already in the messages list
        // Just add the assistant response
        let mut updated_messages = messages; // Use the messages as-is
        updated_messages.push(Message {
            role: "assistant".to_string(),
            content: mock_response.clone(),
        });
        
        // Simulate token counts
        let mock_input_tokens = user_input.len() as u32 / 4;
        let mock_output_tokens = mock_response.len() as u32 / 4;
        
        Ok((mock_response, mock_input_tokens, mock_output_tokens, updated_messages))
    } else {
        // Real API - messages already includes the user message
        let request = ApiRequest {
            model: model.clone(),
            max_tokens,
            temperature,
            messages: messages.clone(), // Use messages as-is
        };

        let client_http = Client::new();
        let response = client_http
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &api_key)
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

        let total_input_tokens = api_response.usage.input_tokens;
        let total_output_tokens = api_response.usage.output_tokens;

        let assistant_response = api_response
            .content
            .iter()
            .filter(|block| block.content_type == "text")
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        // Add only the assistant response
        let mut updated_messages = messages;
        updated_messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_response.clone(),
        });

        Ok((assistant_response, total_input_tokens, total_output_tokens, updated_messages))
    }
}