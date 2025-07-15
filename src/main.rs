mod api;
mod client;
mod syntax;
mod tui;
mod app;
mod config;
mod utils;
mod handlers;
mod ui;

// Test modules
#[cfg(test)]
mod config_utils_tests;
#[cfg(test)]
mod main_loop_tests;
#[cfg(test)]
mod config_color_tests;
#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod config_module_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod main_tests;

use anyhow::Result;
use clap::Parser;
use api::Message;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use tokio::sync::mpsc;
use config::{Args, ColorConfig, SCROLL_ON_USER_INPUT, SCROLL_ON_API_RESPONSE};
use std::time::Duration;
use ui::{layout::create_main_layout, render::draw_ui};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup TUI
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create color configuration from command line arguments
    let (color_result, config_error) = ColorConfig::from_args_and_saved(&args);
    let colors = color_result?;

    // Initialize app state
    let mut app = app::AppState::new(
        args.api_key,
        args.model,
        args.max_tokens,
        args.temperature,
        args.simulate,
        colors,
    )?;
    
    // Show config error dialog if there was an issue loading the config
    if let Some(error_msg) = config_error {
        app.show_config_error(error_msg);
    }

    // Channel for API responses
    let (tx, mut rx) = mpsc::channel::<Result<(String, u32, u32, Vec<Message>), String>>(10);

    loop {
        // Check for new messages BEFORE drawing
        let current_message_count = app.client.messages.len();
        if current_message_count != app.last_message_count {
            let is_user_message = app.client.messages.last()
                .map(|m| m.role == "user")
                .unwrap_or(false);
            
            app.last_message_count = current_message_count;
            
            // Apply feature flags to control when to enable auto-scroll
            if (is_user_message && SCROLL_ON_USER_INPUT) || 
               (!is_user_message && SCROLL_ON_API_RESPONSE) {
                app.auto_scroll = true;
            }
            
            // Mark for redraw since messages changed
            app.mark_dirty();
        }

        // Check if waiting state changed (for progress animation)
        let needs_animation_update = app.waiting;
        if needs_animation_update {
            // Slow down progress animation - only increment every 4th iteration
            static mut FRAME_COUNTER: u32 = 0;
            unsafe {
                FRAME_COUNTER += 1;
                if FRAME_COUNTER % 4 == 0 {
                    app.progress_i += 1;
                    app.mark_dirty(); // Mark for redraw when progress changes
                }
            }
        }

        // Only draw UI if something has changed
        if app.take_dirty() {
            terminal.draw(|f| {
                let size = f.size();
                let layout = create_main_layout(size);
                draw_ui(f, &mut app, &layout);
            })?;
        }

        // Event handling
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    let should_exit = handlers::events::handle_key_event(
                        &mut app,
                        key_event,
                        &tx,
                        (terminal.size()?.width, terminal.size()?.height),
                    ).await?;
                    
                    // Mark for redraw after handling key events
                    app.mark_dirty();
                    
                    if should_exit {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, need to redraw
                    app.mark_dirty();
                }
                _ => {}
            }
        }

        // Check for API responses
        if let Ok(result) = rx.try_recv() {
            app.waiting = false;
            app.status = "Ready".to_string();
            
            match result {
                Ok((_response, input_tokens, output_tokens, updated_messages)) => {
                    // Normal response handling
                    if let Some(assistant_msg) = updated_messages.last() {
                        if assistant_msg.role == "assistant" {
                            app.client.messages.push(assistant_msg.clone());
                        }
                    }
                    
                    app.client.total_input_tokens += input_tokens;
                    app.client.total_output_tokens += output_tokens;
                }
                Err(error_msg) => {
                    // Show the actual error message
                    app.show_error_dialog = true;
                    app.error_message = error_msg;
                }
            }
            
            // Mark for redraw after API response
            app.mark_dirty();
        }

        // Add a small delay to prevent excessive CPU usage
        if app.waiting {
            tokio::time::sleep(Duration::from_millis(10)).await;
        } else {
            // Longer delay when not waiting to reduce CPU usage
            tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS max
        }
    }

    // Save color configuration before cleanup
    if let Err(e) = app.save_color_config() {
        eprintln!("Warning: Failed to save color configuration: {}", e);
    }

    // Cleanup: leave alternate screen and disable raw mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}