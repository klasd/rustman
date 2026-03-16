use crate::models::{ApiResponse, Connection, EditField, InputMode};
use anyhow::Result;
use std::fs;
use tokio::sync::oneshot;

pub struct App {
    pub connections: Vec<Connection>,
    pub selected_connection: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub response: Option<ApiResponse>,
    pub scroll_response: u16,
    pub error_message: Option<String>,
    pub error_message_frames: u32,
    pub edit_field: EditField,
    pub request_cancel_tx: Option<oneshot::Sender<()>>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            connections: Vec::new(),
            selected_connection: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            response: None,
            scroll_response: 0,
            error_message: None,
            error_message_frames: 0,
            edit_field: EditField::Name,
            request_cancel_tx: None,
        };

        // Load all saved connections from JSON files
        app.load_all_connections();

        app
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn delete_selected_connection(&mut self) {
        if !self.connections.is_empty() && self.selected_connection < self.connections.len() {
            self.connections.remove(self.selected_connection);
            if self.selected_connection > 0 {
                self.selected_connection -= 1;
            }
        }
    }

    pub fn save_connection(&self, connection: &Connection) -> Result<()> {
        let json = serde_json::to_string_pretty(connection)?;
        fs::write(format!("{}.json", connection.name), json)?;
        Ok(())
    }

    pub fn load_connection(name: &str) -> Result<Connection> {
        let json = fs::read_to_string(format!("{}.json", name))?;
        let connection = serde_json::from_str(&json)?;
        Ok(connection)
    }

    pub fn load_all_connections(&mut self) {
        // Scan current directory for .json files and load them as connections
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".json") {
                                let connection_name = filename.trim_end_matches(".json");
                                if let Ok(connection) = Self::load_connection(connection_name) {
                                    self.connections.push(connection);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort connections by name for consistent ordering
        self.connections.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn select_next(&mut self) {
        if !self.connections.is_empty() {
            self.selected_connection = (self.selected_connection + 1) % self.connections.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.connections.is_empty() {
            self.selected_connection = if self.selected_connection == 0 {
                self.connections.len() - 1
            } else {
                self.selected_connection - 1
            };
        }
    }

    pub fn current_connection(&self) -> Option<&Connection> {
        self.connections.get(self.selected_connection)
    }

    pub fn current_connection_mut(&mut self) -> Option<&mut Connection> {
        self.connections.get_mut(self.selected_connection)
    }

    pub fn tick(&mut self) {
        if self.error_message_frames > 0 {
            self.error_message_frames -= 1;
        } else if self.error_message_frames == 0 && self.error_message.is_some() {
            self.error_message = None;
        }
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_message_frames = 30; // Display for ~3 seconds at 10 ticks/sec
    }

    pub fn cancel_request(&mut self) {
        if let Some(tx) = self.request_cancel_tx.take() {
            let _ = tx.send(());
        }
        self.input_mode = InputMode::Normal;
        self.set_error("Request cancelled".to_string());
    }
}
