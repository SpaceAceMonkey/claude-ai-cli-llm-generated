use serde::{Deserialize, Serialize};
use ratatui::text::Line;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Cache for highlighted content
#[derive(Debug, Clone)]
pub struct HighlightCache {
    // Maps message content hash to highlighted lines
    cache: HashMap<u64, Vec<Line<'static>>>,
    // Maximum number of entries before we start evicting
    max_size: usize,
}

impl HighlightCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 100, // Keep up to 100 cached messages
        }
    }
    
    pub fn get(&self, content_hash: u64) -> Option<&Vec<Line<'static>>> {
        self.cache.get(&content_hash)
    }
    
    pub fn insert(&mut self, content_hash: u64, lines: Vec<Line<'static>>) {
        // If we're at capacity, remove some entries
        if self.cache.len() >= self.max_size {
            // Remove the oldest 25% of entries (simple LRU alternative)
            let keys_to_remove: Vec<u64> = self.cache.keys()
                .take(self.max_size / 4)
                .copied()
                .collect();
            
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
        
        self.cache.insert(content_hash, lines);
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}

#[derive(Serialize, Debug)]
pub struct ApiRequest {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ApiResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Deserialize, Debug)]
pub struct ErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}