// src/app.rs
use crate::client::ConversationClient;
use crate::api::HighlightCache;
use crate::config::ColorConfig;
use rustyline::Editor;
use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::handlers::file_ops::get_saves_directory;

pub struct AppState {
    pub client: ConversationClient,
    pub input: String,
    pub status: String,
    pub waiting: bool,
    pub progress_i: usize,
    pub history_index: Option<usize>,
    pub chat_scroll_offset: u16,
    pub auto_scroll: bool,
    pub last_message_count: usize,
    pub cursor_position: usize,
    pub input_scroll_offset: u16,
    pub input_draft: Option<String>,
    pub simulate_mode: bool,
    pub rl: Editor<(), rustyline::history::DefaultHistory>,
    
    // Highlighting cache
    pub highlight_cache: HighlightCache,
    
    // Dialog state
    pub show_error_dialog: bool,
    pub error_message: String,
    pub show_save_dialog: bool,
    pub show_load_dialog: bool,
    pub save_filename: String,
    pub available_files: Vec<String>,
    pub file_list_state: ListState,
    pub dialog_cursor_pos: usize,
    pub current_directory: PathBuf,
    pub show_create_dir_dialog: bool,
    pub new_dir_name: String,
    pub show_exit_dialog: bool,
    pub exit_selected: usize,
    
    // Color configuration
    pub colors: ColorConfig,
    pub show_color_dialog: bool,
    pub color_dialog_selection: usize,
    pub color_dialog_option: usize,
    pub color_dialog_scroll_offset: usize,
    pub color_dialog_selection_scroll_offset: usize,
    
    // Color profile management
    pub show_profile_dialog: bool,
    pub profile_dialog_selection: usize,
    pub profile_dialog_scroll_offset: usize,
    pub available_profiles: HashMap<String, crate::config::ColorProfile>,
}

impl AppState {
    pub fn new(
        api_key: String,
        model: String,
        max_tokens: u32,
        temperature: f32,
        simulate_mode: bool,
        colors: ColorConfig,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client: ConversationClient::new(api_key, model, max_tokens, temperature),
            input: String::new(),
            status: String::new(),
            waiting: false,
            progress_i: 0,
            history_index: None,
            chat_scroll_offset: 0,
            auto_scroll: true,
            last_message_count: 0,
            cursor_position: 0,
            input_scroll_offset: 0,
            input_draft: None,
            simulate_mode,
            rl: Editor::<(), rustyline::history::DefaultHistory>::new()?,
            
            // Highlighting cache
            highlight_cache: HighlightCache::new(),
            
            // Dialog state
            show_error_dialog: false,
            error_message: String::new(),
            show_save_dialog: false,
            show_load_dialog: false,
            save_filename: String::new(),
            available_files: Vec::new(),
            file_list_state: ListState::default(),
            dialog_cursor_pos: 0,
            current_directory: get_saves_directory(),
            show_create_dir_dialog: false,
            new_dir_name: String::new(),
            show_exit_dialog: false,
            exit_selected: 0,
            
            // Color configuration
            colors,
            show_color_dialog: false,
            color_dialog_selection: 0,
            color_dialog_option: 0,
            color_dialog_scroll_offset: 0,
            color_dialog_selection_scroll_offset: 0,
            
            // Color profile management
            show_profile_dialog: false,
            profile_dialog_selection: 0,
            profile_dialog_scroll_offset: 0,
            available_profiles: crate::config::get_all_profiles(),
        })
    }
    
    /// Clear the highlight cache when the conversation is cleared or changed
    pub fn clear_highlight_cache(&mut self) {
        self.highlight_cache.clear();
    }
    
    /// Show error dialog for config loading issues
    pub fn show_config_error(&mut self, error_msg: String) {
        self.show_error_dialog = true;
        self.error_message = error_msg;
    }
    
    /// Save current color configuration to disk
    pub fn save_color_config(&self) -> anyhow::Result<()> {
        crate::config::save_color_config(&self.colors)
    }
}