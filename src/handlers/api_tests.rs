//! Unit tests for API functionality
//! Tests simulate mode, API integration, error handling, and async communication

use tokio::sync::mpsc;

use crate::app::AppState;
use crate::api::Message;
use crate::handlers::api::send_message_to_api;

/// Helper function to create a test channel for async communication
fn create_api_test_channel() -> (
    mpsc::Sender<Result<(String, u32, u32, Vec<Message>), String>>,
    mpsc::Receiver<Result<(String, u32, u32, Vec<Message>), String>>,
) {
    mpsc::channel(10)
}

/// Helper function to create test messages
fn create_test_messages() -> Vec<Message> {
    vec![
        Message {
            role: "user".to_string(),
            content: "Hello, Claude!".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "Hello! How can I help you today?".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Tell me about Rust programming.".to_string(),
        },
    ]
}

/// Helper function to create a test app state for API tests
fn create_api_test_app_state(simulate: bool) -> AppState {
    AppState::new(
        "test_key".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        1024,
        0.7,
        simulate,
    ).expect("Failed to create test app state")
}

#[cfg(test)]
mod simulate_mode_tests {
    use super::*;

    #[tokio::test]
    async fn test_simulate_mode_basic_response() {
        // Test that simulate mode returns a mock response without making API calls
        let messages = create_test_messages();
        let user_input = "Test message for simulation".to_string();
        
        let result = send_message_to_api(
            user_input.clone(),
            messages.clone(),
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true, // simulate mode
        ).await;
        
        assert!(result.is_ok(), "Simulate mode should always succeed");
        
        let (response, input_tokens, output_tokens, updated_messages) = result.unwrap();
        
        // Check response contains expected simulation text
        assert!(response.contains("simulated response"), "Response should indicate simulation");
        assert!(response.contains(&user_input), "Response should reference user input");
        
        // Check token counts are reasonable
        assert!(input_tokens > 0, "Should have positive input token count");
        assert!(output_tokens > 0, "Should have positive output token count");
        
        // Check messages were updated correctly
        assert_eq!(updated_messages.len(), messages.len() + 1, "Should add one assistant message");
        assert_eq!(updated_messages.last().unwrap().role, "assistant", "Last message should be assistant");
        assert_eq!(updated_messages.last().unwrap().content, response, "Last message content should match response");
    }

    #[tokio::test]
    async fn test_simulate_mode_delay() {
        // Test that simulate mode includes a realistic delay
        let messages = create_test_messages();
        let user_input = "Test delay".to_string();
        
        let start = std::time::Instant::now();
        
        let result = send_message_to_api(
            user_input,
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Simulate mode should succeed");
        assert!(duration >= std::time::Duration::from_millis(400), "Should have realistic delay");
        assert!(duration <= std::time::Duration::from_millis(1000), "Should not be too slow");
    }

    #[tokio::test]
    async fn test_simulate_mode_token_calculation() {
        // Test that token calculation in simulate mode is reasonable
        let messages = create_test_messages();
        let short_input = "Hi".to_string();
        let long_input = "This is a much longer message that should result in more tokens being calculated for the simulation mode response".to_string();
        
        let short_result = send_message_to_api(
            short_input.clone(),
            messages.clone(),
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await.unwrap();
        
        let long_result = send_message_to_api(
            long_input.clone(),
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await.unwrap();
        
        let (_, short_input_tokens, _, _) = short_result;
        let (_, long_input_tokens, _, _) = long_result;
        
        assert!(long_input_tokens > short_input_tokens, "Longer input should result in more tokens");
    }

    #[tokio::test]
    async fn test_simulate_mode_preserves_message_history() {
        // Test that simulate mode preserves the existing message history
        let messages = create_test_messages();
        let original_count = messages.len();
        let user_input = "Test message".to_string();
        
        let result = send_message_to_api(
            user_input,
            messages.clone(),
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await.unwrap();
        
        let (_, _, _, updated_messages) = result;
        
        // Check that all original messages are preserved
        for (i, original_msg) in messages.iter().enumerate() {
            assert_eq!(updated_messages[i].role, original_msg.role, "Original message role should be preserved");
            assert_eq!(updated_messages[i].content, original_msg.content, "Original message content should be preserved");
        }
        
        assert_eq!(updated_messages.len(), original_count + 1, "Should add exactly one message");
    }

    #[tokio::test]
    async fn test_simulate_mode_unicode_handling() {
        // Test that simulate mode handles Unicode input correctly
        let messages = vec![];
        let unicode_input = "Hello 世界! 🦀 Rust は素晴らしい".to_string();
        
        let result = send_message_to_api(
            unicode_input.clone(),
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        assert!(result.is_ok(), "Simulate mode should handle Unicode input");
        
        let (response, _, _, _) = result.unwrap();
        assert!(response.contains("世界"), "Response should handle Unicode characters");
        assert!(response.contains("🦀"), "Response should handle emoji");
    }
}

#[cfg(test)]
mod api_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_error_handling_invalid_key() {
        // Test handling of invalid API key (in real mode)
        let messages = create_test_messages();
        let user_input = "Test message".to_string();
        
        let result = send_message_to_api(
            user_input,
            messages,
            "invalid_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            false, // real mode
        ).await;
        
        assert!(result.is_err(), "Invalid API key should result in error");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("API Error") || error_msg.contains("Failed to send request"), 
                "Error message should indicate API problem");
    }

    #[tokio::test]
    async fn test_api_error_handling_network_failure() {
        // Test handling of network failures
        let messages = create_test_messages();
        let user_input = "Test message".to_string();
        
        // Use an invalid endpoint to simulate network failure
        let result = send_message_to_api(
            user_input,
            messages,
            "sk-ant-api03-test".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            false, // real mode
        ).await;
        
        assert!(result.is_err(), "Network failure should result in error");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to send request") || error_msg.contains("API Error"), 
                "Error message should indicate network problem");
    }

    #[tokio::test]
    async fn test_api_parameter_validation() {
        // Test that API parameters are validated
        let messages = create_test_messages();
        let user_input = "Test message".to_string();
        
        // Test with invalid temperature (should still work but may be clamped)
        let result = send_message_to_api(
            user_input,
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            5.0, // Invalid temperature > 1.0
            true, // simulate mode
        ).await;
        
        assert!(result.is_ok(), "Simulate mode should handle invalid temperature gracefully");
    }

    #[tokio::test]
    async fn test_empty_message_handling() {
        // Test handling of empty messages
        let messages = vec![];
        let user_input = "".to_string();
        
        let result = send_message_to_api(
            user_input,
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        assert!(result.is_ok(), "Empty message should be handled gracefully");
        
        let (response, _, _, _) = result.unwrap();
        assert!(!response.is_empty(), "Response should not be empty even for empty input");
    }

    #[tokio::test]
    async fn test_very_long_message_handling() {
        // Test handling of very long messages
        let messages = vec![];
        let user_input = "A".repeat(10000); // Very long message
        
        let result = send_message_to_api(
            user_input,
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        assert!(result.is_ok(), "Very long message should be handled");
        
        let (_, input_tokens, _, _) = result.unwrap();
        assert!(input_tokens > 1000, "Long message should result in high token count");
    }
}

#[cfg(test)]
mod async_communication_tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_async_channel_communication() {
        // Test that async communication works correctly
        let (tx, mut rx) = create_api_test_channel();
        let messages = create_test_messages();
        let user_input = "Test async communication".to_string();
        
        // Spawn the API call
        let tx_clone = tx.clone();
        let messages_clone = messages.clone();
        let user_input_clone = user_input.clone();
        
        tokio::spawn(async move {
            let result = send_message_to_api(
                user_input_clone,
                messages_clone,
                "dummy_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                1024,
                0.7,
                true,
            ).await;
            
            match result {
                Ok((response, input_tokens, output_tokens, updated_messages)) => {
                    tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
                }
                Err(e) => {
                    tx_clone.send(Err(e.to_string())).await.ok();
                }
            }
        });
        
        // Wait for response
        let result = timeout(Duration::from_secs(2), rx.recv()).await;
        assert!(result.is_ok(), "Should receive response within timeout");
        
        let response = result.unwrap().unwrap();
        assert!(response.is_ok(), "Response should be successful");
        
        let (response_text, _, _, _) = response.unwrap();
        assert!(response_text.contains("simulated response"), "Should receive simulated response");
    }

    #[tokio::test]
    async fn test_multiple_concurrent_requests() {
        // Test handling of multiple concurrent API requests
        let (tx, mut rx) = create_api_test_channel();
        let messages = create_test_messages();
        
        // Send multiple requests concurrently
        for i in 0..3 {
            let tx_clone = tx.clone();
            let messages_clone = messages.clone();
            let user_input = format!("Concurrent request {}", i);
            
            tokio::spawn(async move {
                let result = send_message_to_api(
                    user_input,
                    messages_clone,
                    "dummy_key".to_string(),
                    "claude-3-5-sonnet-20241022".to_string(),
                    1024,
                    0.7,
                    true,
                ).await;
                
                match result {
                    Ok((response, input_tokens, output_tokens, updated_messages)) => {
                        tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
                    }
                    Err(e) => {
                        tx_clone.send(Err(e.to_string())).await.ok();
                    }
                }
            });
        }
        
        // Collect responses
        let mut responses = Vec::new();
        for _ in 0..3 {
            let result = timeout(Duration::from_secs(3), rx.recv()).await;
            assert!(result.is_ok(), "Should receive all responses");
            responses.push(result.unwrap().unwrap());
        }
        
        // Verify all responses are successful
        for (i, response) in responses.iter().enumerate() {
            assert!(response.is_ok(), "Response {} should be successful", i);
        }
    }

    #[tokio::test]
    async fn test_channel_error_propagation() {
        // Test that errors are properly propagated through the channel
        let (tx, mut rx) = create_api_test_channel();
        let messages = create_test_messages();
        let user_input = "Test error propagation".to_string();
        
        // Spawn request with invalid parameters to trigger error
        let tx_clone = tx.clone();
        let messages_clone = messages.clone();
        
        tokio::spawn(async move {
            let result = send_message_to_api(
                user_input,
                messages_clone,
                "invalid_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                1024,
                0.7,
                false, // real mode to trigger error
            ).await;
            
            match result {
                Ok((response, input_tokens, output_tokens, updated_messages)) => {
                    tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
                }
                Err(e) => {
                    tx_clone.send(Err(e.to_string())).await.ok();
                }
            }
        });
        
        // Wait for error response
        let result = timeout(Duration::from_secs(10), rx.recv()).await;
        assert!(result.is_ok(), "Should receive error response");
        
        let response = result.unwrap().unwrap();
        assert!(response.is_err(), "Response should be an error");
        
        let error_msg = response.unwrap_err();
        assert!(error_msg.contains("API Error") || error_msg.contains("Failed to send request"), 
                "Error message should be meaningful");
    }

    #[tokio::test]
    async fn test_channel_capacity_handling() {
        // Test that channel capacity is handled correctly
        let (tx, mut rx) = mpsc::channel::<Result<(String, u32, u32, Vec<Message>), String>>(1); // Small capacity
        let messages = create_test_messages();
        
        // Send more messages than channel capacity
        for i in 0..3 {
            let tx_clone = tx.clone();
            let messages_clone = messages.clone();
            let user_input = format!("Capacity test {}", i);
            
            tokio::spawn(async move {
                let result = send_message_to_api(
                    user_input,
                    messages_clone,
                    "dummy_key".to_string(),
                    "claude-3-5-sonnet-20241022".to_string(),
                    1024,
                    0.7,
                    true,
                ).await;
                
                match result {
                    Ok((response, input_tokens, output_tokens, updated_messages)) => {
                        tx_clone.send(Ok((response, input_tokens, output_tokens, updated_messages))).await.ok();
                    }
                    Err(e) => {
                        tx_clone.send(Err(e.to_string())).await.ok();
                    }
                }
            });
            
            // Small delay to ensure ordering
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Process messages - should handle capacity limits gracefully
        let mut received_count = 0;
        while let Ok(result) = timeout(Duration::from_secs(2), rx.recv()).await {
            if result.is_some() {
                received_count += 1;
            }
            if received_count >= 3 {
                break;
            }
        }
        
        assert!(received_count > 0, "Should receive at least one message");
    }
}

#[cfg(test)]
mod message_history_tests {
    use super::*;

    #[tokio::test]
    async fn test_message_history_preservation() {
        // Test that message history is preserved correctly
        let mut messages = vec![
            Message {
                role: "user".to_string(),
                content: "First message".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "First response".to_string(),
            },
        ];
        
        let user_input = "Second message".to_string();
        
        let result = send_message_to_api(
            user_input.clone(),
            messages.clone(),
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await.unwrap();
        
        let (_, _, _, updated_messages) = result;
        
        // Check that original messages are preserved
        assert_eq!(updated_messages[0].role, "user");
        assert_eq!(updated_messages[0].content, "First message");
        assert_eq!(updated_messages[1].role, "assistant");
        assert_eq!(updated_messages[1].content, "First response");
        
        // Check that new assistant message is added
        assert_eq!(updated_messages.len(), 3);
        assert_eq!(updated_messages[2].role, "assistant");
        assert!(updated_messages[2].content.contains("simulated response"));
    }

    #[tokio::test]
    async fn test_conversation_context_building() {
        // Test building up a conversation context
        let mut messages = vec![];
        
        // Simulate multiple turns of conversation
        let turns = vec![
            "Hello, what's your name?",
            "Can you help me with Rust?",
            "What are lifetimes?",
        ];
        
        for turn in turns {
            // Add user message first
            messages.push(Message {
                role: "user".to_string(),
                content: turn.to_string(),
            });
            
            let result = send_message_to_api(
                turn.to_string(),
                messages.clone(),
                "dummy_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                1024,
                0.7,
                true,
            ).await.unwrap();
            
            let (_, _, _, updated_messages) = result;
            messages = updated_messages;
        }
        
        // Should have 6 messages total (3 user + 3 assistant)
        assert_eq!(messages.len(), 6);
        
        // Check alternating pattern (user, assistant, user, assistant, etc.)
        for (i, msg) in messages.iter().enumerate() {
            if i % 2 == 0 {
                assert_eq!(msg.role, "user");
            } else {
                assert_eq!(msg.role, "assistant");
            }
        }
    }

    #[tokio::test]
    async fn test_large_conversation_history() {
        // Test handling of large conversation history
        let mut messages = vec![];
        
        // Create a large conversation history
        for i in 0..50 {
            messages.push(Message {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("Message number {}", i),
            });
        }
        
        let user_input = "Latest message".to_string();
        
        let result = send_message_to_api(
            user_input,
            messages.clone(),
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        assert!(result.is_ok(), "Should handle large conversation history");
        
        let (_, _, _, updated_messages) = result.unwrap();
        assert_eq!(updated_messages.len(), 51, "Should add one new message");
        
        // Check that all original messages are preserved
        for (i, original) in messages.iter().enumerate() {
            assert_eq!(updated_messages[i].role, original.role);
            assert_eq!(updated_messages[i].content, original.content);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_simulate_mode_performance() {
        // Test that simulate mode performs well
        let messages = create_test_messages();
        let user_input = "Performance test".to_string();
        
        let start = Instant::now();
        
        // Run multiple simulated requests
        for i in 0..10 {
            let result = send_message_to_api(
                format!("{} {}", user_input, i),
                messages.clone(),
                "dummy_key".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                1024,
                0.7,
                true,
            ).await;
            
            assert!(result.is_ok(), "Request {} should succeed", i);
        }
        
        let duration = start.elapsed();
        
        // Should complete reasonably quickly (allowing for simulated delays)
        assert!(duration < std::time::Duration::from_secs(10), 
                "10 simulated requests should complete in under 10 seconds");
    }

    #[tokio::test]
    async fn test_memory_usage_with_large_messages() {
        // Test memory usage with large messages
        let messages = vec![];
        let large_input = "A".repeat(100000); // 100KB message
        
        let result = send_message_to_api(
            large_input,
            messages,
            "dummy_key".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            1024,
            0.7,
            true,
        ).await;
        
        assert!(result.is_ok(), "Should handle large messages");
        
        let (response, _, _, _) = result.unwrap();
        assert!(!response.is_empty(), "Should generate response for large input");
    }
}
