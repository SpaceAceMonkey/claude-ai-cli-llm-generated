use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::client::ConversationClient;
use crate::api::Message;

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedConversation {
    pub version: String,
    pub timestamp: String,
    pub model: String,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub messages: Vec<Message>,
}

impl SavedConversation {
    pub fn new(client: &ConversationClient) -> Self {
        Self {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: client.model.clone(),
            total_input_tokens: client.total_input_tokens,
            total_output_tokens: client.total_output_tokens,
            messages: client.messages.clone(),
        }
    }

    pub fn validate(&self) -> bool {
        // Validate the conversation file format - empty messages are OK
        self.version == "1.0"
    }
}

pub fn get_saves_directory() -> PathBuf {
    // Start from current working directory where the executable is
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn load_directory_contents(files: &mut Vec<String>, current_dir: &PathBuf, is_save_dialog: bool) {
    files.clear();
    
    // Add parent directory unless we're at root
    if current_dir.parent().is_some() {
        files.push("../".to_string());
    }
    
    // Add option to create new directory only for save dialog
    if is_save_dialog {
        files.push("[ Create New Directory ]".to_string());
    }
    
    if let Ok(entries) = fs::read_dir(current_dir) {
        let mut dirs = Vec::new();
        let mut regular_files = Vec::new();
        
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                // Show hidden directories starting with '.' but skip hidden files
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(format!("{}/", filename));
                } else if !filename.starts_with('.') {
                    regular_files.push(filename.to_string());
                }
            }
        }
        
        // Sort directories and files separately
        dirs.sort();
        regular_files.sort();
        
        // Add directories first, then files
        files.extend(dirs);
        files.extend(regular_files);
    }
    
    // If directory is empty, show a message
    let expected_count = if current_dir.parent().is_some() { 1 } else { 0 } + if is_save_dialog { 1 } else { 0 };
    if files.len() <= expected_count {
        files.push("(Empty directory)".to_string());
    }
}

pub fn save_conversation(client: &ConversationClient, filepath: &PathBuf) -> Result<()> {
    let conversation = SavedConversation::new(client);
    let json = serde_json::to_string_pretty(&conversation)?;
    fs::write(filepath, json)?;
    Ok(())
}

pub fn load_conversation(filepath: &PathBuf) -> Result<SavedConversation> {
    let json = fs::read_to_string(filepath)?;
    let conversation: SavedConversation = serde_json::from_str(&json)?;
    if !conversation.validate() {
        return Err(anyhow::anyhow!("Invalid conversation file format"));
    }
    Ok(conversation)
}
