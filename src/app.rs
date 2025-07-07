// src/app.rs
use crate::client::ConversationClient;
use rustyline::Editor;

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
        })
    }
}