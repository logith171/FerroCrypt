mod crypto;
mod storage;
mod ui;
mod sync;

use crate::storage::{Vault, PasswordEntry, Config};
use crate::crypto::Crypto;
use std::{error::Error, io, path::Path};
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::*};
use ratatui::prelude::*;

/// The three modes our application can be in.
#[derive(PartialEq)]
pub enum AppMode {
    Setup, // First time run: Set Master Pass and GitHub info
    Login, // Returning: Enter Master Pass to unlock
    Main,  // Logged in: Manage passwords
}

pub struct App {
    pub mode: AppMode,
    pub config: Config,
    pub vault: Vault,
    pub master_pass_input: String,
    pub master_key: Option<[u8; 32]>,
    
    // Main Dashboard State
    pub selected_index: usize,
    pub editing: bool,
    pub is_new_entry: bool, // true if 'a', false if 'e'
    pub title_input: String,
    pub user_input: String,
    pub pass_input: String,
    pub active_field: usize, // 0: Title, 1: Username, 2: Password
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Determine if this is a first-time setup or a login
    let mut app = App {
        mode: if Path::new("vault.enc").exists() { AppMode::Login } else { AppMode::Setup },
        config: Vault::load_config(),
        vault: Vault::default(),
        master_pass_input: String::new(),
        master_key: None,
        selected_index: 0,
        editing: false,
        is_new_entry: false,
        title_input: String::new(),
        user_input: String::new(),
        pass_input: String::new(),
        active_field: 0,
    };

    // 2. Setup the Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // 3. Application Loop
    loop {
        // Draw the UI based on current mode/state
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle Keyboard Inputs
        if let Event::Key(key) = event::read()? {
            match app.mode {
                
                // --- SETUP MODE LOGIC ---
                AppMode::Setup => match key.code {
                    KeyCode::Tab => app.active_field = (app.active_field + 1) % 3,
                    KeyCode::Enter => {
                        if !app.master_pass_input.is_empty() {
                            let key = Crypto::derive_key(&app.master_pass_input);
                            app.master_key = Some(key);
                            let _ = Vault::save_config(&app.config);
                            let encrypted = Crypto::encrypt(&app.vault.to_bytes(), &key);
                            let _ = Vault::save_local("vault.enc", &encrypted);
                            app.mode = AppMode::Main;
                            app.master_pass_input.clear();
                        }
                    }
                    KeyCode::Backspace => match app.active_field {
                        0 => { app.master_pass_input.pop(); },
                        1 => { app.config.github_token.pop(); },
                        2 => { app.config.gist_id.pop(); },
                        _ => {}
                    },
                    KeyCode::Char(c) => match app.active_field {
                        0 => app.master_pass_input.push(c),
                        1 => app.config.github_token.push(c),
                        2 => app.config.gist_id.push(c),
                        _ => {}
                    },
                    _ => {}
                },

                // --- LOGIN MODE LOGIC ---
                AppMode::Login => match key.code {
                    KeyCode::Enter => {
                        let key = Crypto::derive_key(&app.master_pass_input);
                        if let Ok(bytes) = Vault::load_local("vault.enc") {
                            if let Some(decrypted) = Crypto::decrypt(&bytes, &key) {
                                app.vault = Vault::from_bytes(&decrypted);
                                app.master_key = Some(key);
                                app.mode = AppMode::Main;
                            }
                        }
                        app.master_pass_input.clear();
                    }
                    KeyCode::Backspace => { app.master_pass_input.pop(); },
                    KeyCode::Char(c) => app.master_pass_input.push(c),
                    _ => {}
                },

                // --- MAIN DASHBOARD LOGIC ---
                AppMode::Main => {
                    if app.editing {
                        // Logic while the Input Form is open
                        match key.code {
                            KeyCode::Esc => app.editing = false,
                            KeyCode::Tab => app.active_field = (app.active_field + 1) % 3,
                            KeyCode::Backspace => match app.active_field {
                                0 => { app.title_input.pop(); },
                                1 => { app.user_input.pop(); },
                                2 => { app.pass_input.pop(); },
                                _ => {}
                            },
                            KeyCode::Char(c) => match app.active_field {
                                0 => app.title_input.push(c),
                                1 => app.user_input.push(c),
                                2 => app.pass_input.push(c),
                                _ => {}
                            },
                            KeyCode::Enter => {
                                let entry = PasswordEntry {
                                    title: app.title_input.clone(),
                                    username: app.user_input.clone(),
                                    password: app.pass_input.clone(),
                                };
                                if app.is_new_entry {
                                    app.vault.entries.push(entry);
                                } else {
                                    app.vault.entries[app.selected_index] = entry;
                                }
                                // Auto-save locally
                                if let Some(key) = app.master_key {
                                    let encrypted = Crypto::encrypt(&app.vault.to_bytes(), &key);
                                    let _ = Vault::save_local("vault.enc", &encrypted);
                                }
                                app.editing = false;
                            }
                            _ => {}
                        }
                    } else {
                        // Logic while navigating the List
                        match key.code {
                            KeyCode::Char('q') => break, // Quit
                            KeyCode::Char('a') => {      // Add new
                                app.editing = true;
                                app.is_new_entry = true;
                                app.active_field = 0;
                                app.title_input.clear();
                                app.user_input.clear();
                                app.pass_input.clear();
                            }
                            KeyCode::Char('e') => {      // Edit existing
                                if let Some(e) = app.vault.entries.get(app.selected_index) {
                                    app.editing = true;
                                    app.is_new_entry = false;
                                    app.active_field = 0;
                                    app.title_input = e.title.clone();
                                    app.user_input = e.username.clone();
                                    app.pass_input = e.password.clone();
                                }
                            }
                            KeyCode::Char('d') => {      // Delete
                                if !app.vault.entries.is_empty() {
                                    app.vault.entries.remove(app.selected_index);
                                    if app.selected_index > 0 { app.selected_index -= 1; }
                                    if let Some(key) = app.master_key {
                                        let encrypted = Crypto::encrypt(&app.vault.to_bytes(), &key);
                                        let _ = Vault::save_local("vault.enc", &encrypted);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if !app.vault.entries.is_empty() {
                                    app.selected_index = (app.selected_index + 1) % app.vault.entries.len();
                                }
                            }
                            KeyCode::Up => {
                                if !app.vault.entries.is_empty() {
                                    app.selected_index = if app.selected_index == 0 { app.vault.entries.len() - 1 } else { app.selected_index - 1 };
                                }
                            }
                            // PUSH to Cloud
                            KeyCode::Char('s') => {
                                let token = app.config.github_token.clone();
                                let gist = app.config.gist_id.clone();
                                if let Some(key) = app.master_key {
                                    let encrypted = Crypto::encrypt(&app.vault.to_bytes(), &key);
                                    tokio::spawn(async move {
                                        let _ = sync::SyncClient::upload_to_gist(&token, &gist, &encrypted).await;
                                    });
                                }
                            }
                            // PULL from Cloud (Multi-machine Case)
                            KeyCode::Char('p') => {
                                let token = app.config.github_token.clone();
                                let gist = app.config.gist_id.clone();
                                let master_key = app.master_key.unwrap();
                                
                                tokio::spawn(async move {
                                    // 1. Download encrypted file from Gist
                                    if let Ok(encrypted_bytes) = sync::SyncClient::download_from_gist(&token, &gist).await {
                                        // 2. Verify decryption works before saving
                                        if crypto::Crypto::decrypt(&encrypted_bytes, &master_key).is_some() {
                                            // 3. Overwrite local file. 
                                            // User restarts app to see fresh data.
                                            let _ = storage::Vault::save_local("vault.enc", &encrypted_bytes);
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // 4. Cleanup Terminal and Exit
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}