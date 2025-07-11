// src/app.rs
use crate::client::ConversationClient;
use rustyline::Editor;
use ratatui::widgets::ListState;
use std::path::PathBuf;
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
}

impl AppState {
    pub fn new(
        api_key: String,
        model: String,
        max_tokens: u32,
        temperature: f32,
        simulate_mode: bool,
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
        })
    }
}