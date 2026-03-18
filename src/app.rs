use crate::models::{
    ActivePanel, ApiResponse, Connection, EditField, InputMode, KeyValueTarget, HTTP_METHODS,
};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
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
    pub active_panel: ActivePanel,
    pub edit_backup: Option<Connection>,
    pub method_index: usize,
    // Payload editor state
    pub payload_lines: Vec<String>,
    pub payload_cursor_row: usize,
    pub payload_cursor_col: usize,
    pub payload_scroll: usize,
    // Key-value editor state (for headers and query params)
    pub kv_target: KeyValueTarget,
    pub kv_items: Vec<(String, String)>,
    pub kv_selected: usize,
    pub kv_editing: Option<KvEditMode>,
    pub kv_scroll: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KvEditMode {
    Key,
    Value,
}

impl App {
    fn get_config_dir() -> Result<PathBuf> {
        let config_dir = if cfg!(target_os = "windows") {
            // On Windows, use AppData/Local/rustman
            if let Some(app_data) = dirs::data_local_dir() {
                app_data.join("rustman").join("sites")
            } else {
                anyhow::bail!("Could not determine AppData directory");
            }
        } else {
            // On Unix/Linux/macOS, use ~/.config/rustman/sites
            if let Some(config) = dirs::config_dir() {
                config.join("rustman").join("sites")
            } else {
                anyhow::bail!("Could not determine config directory");
            }
        };

        // Create the directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

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
            active_panel: ActivePanel::Connections,
            edit_backup: None,
            method_index: 0,
            payload_lines: vec![String::new()],
            payload_cursor_row: 0,
            payload_cursor_col: 0,
            payload_scroll: 0,
            kv_target: KeyValueTarget::Headers,
            kv_items: Vec::new(),
            kv_selected: 0,
            kv_editing: None,
            kv_scroll: 0,
        };

        // Load all saved connections from JSON files
        app.load_all_connections();

        app
    }

    pub fn add_connection(&mut self, connection: Connection) {
        let name = connection.name.clone();
        self.connections.push(connection);
        // Sort connections by name for consistent ordering
        self.connections.sort_by(|a, b| a.name.cmp(&b.name));
        // Select the newly added connection
        if let Some(idx) = self.connections.iter().position(|c| c.name == name) {
            self.selected_connection = idx;
        }
    }

    pub fn delete_selected_connection(&mut self) {
        if !self.connections.is_empty() && self.selected_connection < self.connections.len() {
            let connection = self.connections.remove(self.selected_connection);

            // Delete the connection file from disk
            if let Ok(config_dir) = Self::get_config_dir() {
                let file_path = config_dir.join(format!("{}.json", connection.name));
                let _ = fs::remove_file(file_path); // Ignore errors if file doesn't exist
            }

            if self.selected_connection > 0 {
                self.selected_connection -= 1;
            }
        }
    }

    pub fn save_connection(&self, connection: &Connection) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let file_path = config_dir.join(format!("{}.json", connection.name));
        let json = serde_json::to_string_pretty(connection)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn delete_connection_file(&self, name: &str) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let file_path = config_dir.join(format!("{}.json", name));
        fs::remove_file(file_path)?;
        Ok(())
    }

    pub fn load_connection(name: &str) -> Result<Connection> {
        let config_dir = Self::get_config_dir()?;
        let file_path = config_dir.join(format!("{}.json", name));
        let json = fs::read_to_string(file_path)?;
        let connection = serde_json::from_str(&json)?;
        Ok(connection)
    }

    pub fn load_all_connections(&mut self) {
        // Scan config directory for .json files and load them as connections
        if let Ok(config_dir) = Self::get_config_dir() {
            if let Ok(entries) = fs::read_dir(config_dir) {
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

    pub fn method_index_from_string(method: &str) -> usize {
        HTTP_METHODS
            .iter()
            .position(|&m| m == method.to_uppercase())
            .unwrap_or(0)
    }

    pub fn current_method(&self) -> &'static str {
        HTTP_METHODS[self.method_index]
    }

    pub fn next_method(&mut self) {
        self.method_index = (self.method_index + 1) % HTTP_METHODS.len();
    }

    pub fn prev_method(&mut self) {
        self.method_index = if self.method_index == 0 {
            HTTP_METHODS.len() - 1
        } else {
            self.method_index - 1
        };
    }

    // Payload editor helpers
    pub fn init_payload_editor(&mut self, payload: Option<&str>) {
        let content = payload.unwrap_or("");
        self.payload_lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        if self.payload_lines.is_empty() {
            self.payload_lines.push(String::new());
        }
        self.payload_cursor_row = 0;
        self.payload_cursor_col = 0;
        self.payload_scroll = 0;
    }

    pub fn payload_to_string(&self) -> Option<String> {
        let content = self.payload_lines.join("\n");
        if content.trim().is_empty() {
            None
        } else {
            Some(content)
        }
    }

    pub fn payload_insert_char(&mut self, c: char) {
        if let Some(line) = self.payload_lines.get_mut(self.payload_cursor_row) {
            // Ensure cursor_col is within bounds
            let col = self.payload_cursor_col.min(line.len());
            line.insert(col, c);
            self.payload_cursor_col = col + 1;
        }
    }

    pub fn payload_backspace(&mut self) {
        if self.payload_cursor_col > 0 {
            if let Some(line) = self.payload_lines.get_mut(self.payload_cursor_row) {
                let col = self.payload_cursor_col.min(line.len());
                if col > 0 {
                    line.remove(col - 1);
                    self.payload_cursor_col = col - 1;
                }
            }
        } else if self.payload_cursor_row > 0 {
            // Merge with previous line
            let current_line = self.payload_lines.remove(self.payload_cursor_row);
            self.payload_cursor_row -= 1;
            if let Some(prev_line) = self.payload_lines.get_mut(self.payload_cursor_row) {
                self.payload_cursor_col = prev_line.len();
                prev_line.push_str(&current_line);
            }
        }
    }

    pub fn payload_delete(&mut self) {
        if let Some(line) = self.payload_lines.get_mut(self.payload_cursor_row) {
            let col = self.payload_cursor_col.min(line.len());
            if col < line.len() {
                line.remove(col);
            } else if self.payload_cursor_row + 1 < self.payload_lines.len() {
                // Merge next line into current
                let next_line = self.payload_lines.remove(self.payload_cursor_row + 1);
                if let Some(current_line) = self.payload_lines.get_mut(self.payload_cursor_row) {
                    current_line.push_str(&next_line);
                }
            }
        }
    }

    pub fn payload_newline(&mut self) {
        if let Some(line) = self.payload_lines.get_mut(self.payload_cursor_row) {
            let col = self.payload_cursor_col.min(line.len());
            let rest = line[col..].to_string();
            line.truncate(col);
            self.payload_cursor_row += 1;
            self.payload_lines.insert(self.payload_cursor_row, rest);
            self.payload_cursor_col = 0;
        }
    }

    pub fn payload_move_up(&mut self) {
        if self.payload_cursor_row > 0 {
            self.payload_cursor_row -= 1;
            // Clamp column to line length
            if let Some(line) = self.payload_lines.get(self.payload_cursor_row) {
                self.payload_cursor_col = self.payload_cursor_col.min(line.len());
            }
        }
    }

    pub fn payload_move_down(&mut self) {
        if self.payload_cursor_row + 1 < self.payload_lines.len() {
            self.payload_cursor_row += 1;
            // Clamp column to line length
            if let Some(line) = self.payload_lines.get(self.payload_cursor_row) {
                self.payload_cursor_col = self.payload_cursor_col.min(line.len());
            }
        }
    }

    pub fn payload_move_left(&mut self) {
        if self.payload_cursor_col > 0 {
            self.payload_cursor_col -= 1;
        } else if self.payload_cursor_row > 0 {
            // Move to end of previous line
            self.payload_cursor_row -= 1;
            if let Some(line) = self.payload_lines.get(self.payload_cursor_row) {
                self.payload_cursor_col = line.len();
            }
        }
    }

    pub fn payload_move_right(&mut self) {
        if let Some(line) = self.payload_lines.get(self.payload_cursor_row) {
            if self.payload_cursor_col < line.len() {
                self.payload_cursor_col += 1;
            } else if self.payload_cursor_row + 1 < self.payload_lines.len() {
                // Move to start of next line
                self.payload_cursor_row += 1;
                self.payload_cursor_col = 0;
            }
        }
    }

    pub fn payload_move_home(&mut self) {
        self.payload_cursor_col = 0;
    }

    pub fn payload_move_end(&mut self) {
        if let Some(line) = self.payload_lines.get(self.payload_cursor_row) {
            self.payload_cursor_col = line.len();
        }
    }

    // Key-value editor helpers
    pub fn init_kv_editor(&mut self, target: KeyValueTarget) {
        self.kv_target = target.clone();
        if let Some(conn) = self.current_connection() {
            let map = match target {
                KeyValueTarget::Headers => &conn.headers,
                KeyValueTarget::QueryParams => &conn.query_params,
            };
            self.kv_items = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            // Sort by key for consistent display
            self.kv_items.sort_by(|a, b| a.0.cmp(&b.0));
        } else {
            self.kv_items = Vec::new();
        }
        self.kv_selected = 0;
        self.kv_editing = None;
        self.kv_scroll = 0;
        self.input_buffer.clear();
    }

    pub fn kv_to_hashmap(&self) -> std::collections::HashMap<String, String> {
        self.kv_items.iter().cloned().collect()
    }

    pub fn kv_add_item(&mut self) {
        self.kv_items.push((String::new(), String::new()));
        self.kv_selected = self.kv_items.len() - 1;
        self.kv_editing = Some(KvEditMode::Key);
        self.input_buffer.clear();
    }

    pub fn kv_delete_selected(&mut self) {
        if !self.kv_items.is_empty() && self.kv_selected < self.kv_items.len() {
            self.kv_items.remove(self.kv_selected);
            if self.kv_selected > 0 && self.kv_selected >= self.kv_items.len() {
                self.kv_selected = self.kv_items.len().saturating_sub(1);
            }
        }
    }

    pub fn kv_move_up(&mut self) {
        if self.kv_selected > 0 {
            self.kv_selected -= 1;
        }
    }

    pub fn kv_move_down(&mut self) {
        if self.kv_selected + 1 < self.kv_items.len() {
            self.kv_selected += 1;
        }
    }

    pub fn kv_start_edit_key(&mut self) {
        if let Some((key, _)) = self.kv_items.get(self.kv_selected) {
            self.input_buffer = key.clone();
            self.kv_editing = Some(KvEditMode::Key);
        }
    }

    pub fn kv_start_edit_value(&mut self) {
        if let Some((_, value)) = self.kv_items.get(self.kv_selected) {
            self.input_buffer = value.clone();
            self.kv_editing = Some(KvEditMode::Value);
        }
    }

    pub fn kv_save_edit(&mut self) {
        if let Some(edit_mode) = &self.kv_editing {
            if let Some(item) = self.kv_items.get_mut(self.kv_selected) {
                match edit_mode {
                    KvEditMode::Key => item.0 = self.input_buffer.clone(),
                    KvEditMode::Value => item.1 = self.input_buffer.clone(),
                }
            }
        }
        self.kv_editing = None;
        self.input_buffer.clear();
    }

    pub fn kv_cancel_edit(&mut self) {
        self.kv_editing = None;
        self.input_buffer.clear();
    }
}
