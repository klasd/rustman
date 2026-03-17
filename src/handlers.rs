use crate::app::App;
use crate::models::{Connection, EditField, InputMode, ActivePanel};
use crossterm::event::{KeyCode, KeyEvent};

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key).await,
        InputMode::ConnectionName => handle_connection_name_input(app, key),
        InputMode::EditingConnection => handle_edit_dialog_input(app, key),
        InputMode::Connecting => handle_connecting_mode(app, key),
    }
}

async fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') => {
            app.input_mode = InputMode::ConnectionName;
            app.input_buffer.clear();
            app.set_error("Enter connection name and press Enter".to_string());
        }
        KeyCode::Char('d') => {
            app.delete_selected_connection();
            app.set_error("Connection deleted".to_string());
        }
        KeyCode::Char('e') => {
            if let Some(conn) = app.current_connection() {
                // Save a backup of the current connection
                app.edit_backup = Some(conn.clone());
                app.input_mode = InputMode::EditingConnection;
                app.edit_field = EditField::Name;
                app.input_buffer.clear();
                load_current_field_value(app);
            } else {
                app.set_error("No connection selected".to_string());
            }
        }
        KeyCode::Char('s') => {
            if let Some(conn) = app.current_connection() {
                let conn_clone = conn.clone();
                let conn_name = conn_clone.name.clone();
                match app.save_connection(&conn_clone) {
                    Ok(_) => {
                        app.set_error(format!("✓ Saved: {}.json", conn_name));
                    }
                    Err(e) => {
                        app.set_error(format!("✗ Save failed: {}", e));
                    }
                }
            } else {
                app.set_error("No connection selected".to_string());
            }
        }
        KeyCode::Char('l') => {
            app.set_error("Load feature coming soon".to_string());
        }
        KeyCode::Up => {
            if app.active_panel == ActivePanel::Connections {
                app.select_prev();
            }
        }
        KeyCode::Down => {
            if app.active_panel == ActivePanel::Connections {
                app.select_next();
            }
        }
        KeyCode::Char('j') => {
            // Vim keybinding: scroll down
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = app.scroll_response.saturating_add(1);
            }
        }
        KeyCode::Char('k') => {
            // Vim keybinding: scroll up
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = app.scroll_response.saturating_sub(1);
            }
        }
        KeyCode::Char('r') => {
            // Send request
            if let Some(conn) = app.current_connection() {
                let conn_clone = conn.clone();
                app.set_error(format!("Sending request to {}:{}", conn_clone.url, conn_clone.port));
                send_request(app, &conn_clone).await;
            } else {
                app.set_error("No connection selected".to_string());
            }
        }
        KeyCode::Char('p') => {
            // Switch panels with 'p' key
            app.active_panel = match app.active_panel {
                ActivePanel::Connections => {
                    app.set_error("Switched to Response panel - use j/k to scroll".to_string());
                    ActivePanel::Response
                }
                ActivePanel::Response => {
                    app.set_error("Switched to Connections panel - use ↑↓ to navigate".to_string());
                    ActivePanel::Connections
                }
            };
        }
        KeyCode::Tab => {
            // Switch between panels
            app.active_panel = match app.active_panel {
                ActivePanel::Connections => {
                    app.set_error("Switched to Response panel - use j/k to scroll".to_string());
                    ActivePanel::Response
                }
                ActivePanel::Response => {
                    app.set_error("Switched to Connections panel - use ↑↓ to navigate".to_string());
                    ActivePanel::Connections
                }
            };
        }
        KeyCode::BackTab => {
            // Shift+Tab also switches panels (reverse direction)
            app.active_panel = match app.active_panel {
                ActivePanel::Connections => {
                    app.set_error("Switched to Response panel - use j/k to scroll".to_string());
                    ActivePanel::Response
                }
                ActivePanel::Response => {
                    app.set_error("Switched to Connections panel - use ↑↓ to navigate".to_string());
                    ActivePanel::Connections
                }
            };
        }
        KeyCode::PageDown => {
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = app.scroll_response.saturating_add(5);
            }
        }
        KeyCode::PageUp => {
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = app.scroll_response.saturating_sub(5);
            }
        }
        KeyCode::Home => {
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = 0;
            }
        }
        KeyCode::End => {
            if app.active_panel == ActivePanel::Response {
                app.scroll_response = u16::MAX;
            }
        }
        _ => {}
    }
}

fn handle_connection_name_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Enter => {
            if !app.input_buffer.is_empty() {
                let name = app.input_buffer.clone();
                let conn = Connection::new(name.clone(), "localhost".to_string(), 3000);
                // Save to disk immediately
                if let Err(e) = app.save_connection(&conn) {
                    app.set_error(format!("✗ Failed to create connection: {}", e));
                } else {
                    app.add_connection(conn);
                    app.set_error(format!("✓ Connection '{}' created and saved", name));
                }
            } else {
                app.set_error("Connection name cannot be empty".to_string());
            }
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        _ => {}
    }
}

fn handle_edit_dialog_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            // Save the current field before moving to next
            apply_field_edit(app);
            // Move to next field
            app.edit_field = match app.edit_field {
                EditField::Name => EditField::Url,
                EditField::Url => EditField::Port,
                EditField::Port => EditField::Method,
                EditField::Method => EditField::Name,
            };
            app.input_buffer.clear();
            load_current_field_value(app);
        }
        KeyCode::BackTab => {
            // Save the current field before moving to previous
            apply_field_edit(app);
            // Move to previous field
            app.edit_field = match app.edit_field {
                EditField::Name => EditField::Method,
                EditField::Url => EditField::Name,
                EditField::Port => EditField::Url,
                EditField::Method => EditField::Port,
            };
            app.input_buffer.clear();
            load_current_field_value(app);
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Enter => {
            // Save the current field and exit the dialog
            apply_field_edit(app);
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
            app.edit_backup = None;
            // Save to disk
            if let Some(conn) = app.current_connection() {
                let conn_clone = conn.clone();
                if let Err(e) = app.save_connection(&conn_clone) {
                    app.set_error(format!("✗ Failed to save: {}", e));
                } else {
                    app.set_error("✓ Connection saved".to_string());
                }
            }
        }
        KeyCode::Esc => {
            // Discard changes and restore from backup
            if let Some(backup) = app.edit_backup.take() {
                if let Some(conn) = app.current_connection_mut() {
                    *conn = backup;
                }
            }
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
            app.set_error("✗ Changes discarded".to_string());
        }
        _ => {}
    }
}

fn load_current_field_value(app: &mut App) {
    if let Some(conn) = app.current_connection() {
        match app.edit_field {
            EditField::Name => {
                app.input_buffer = conn.name.clone();
            }
            EditField::Url => {
                app.input_buffer = conn.url.clone();
            }
            EditField::Port => {
                app.input_buffer = conn.port.to_string();
            }
            EditField::Method => {
                app.input_buffer = conn.method.clone();
            }
        }
    }
}

fn apply_field_edit(app: &mut App) {
    let edit_field = app.edit_field.clone();
    let input_buffer = app.input_buffer.clone();
    
    if let Some(conn) = app.current_connection_mut() {
        match edit_field {
            EditField::Name => {
                if !input_buffer.is_empty() {
                    conn.name = input_buffer.clone();
                    app.set_error(format!("✓ Name updated: {}", input_buffer));
                }
            }
            EditField::Url => {
                if !input_buffer.is_empty() {
                    conn.url = input_buffer.clone();
                    app.set_error(format!("✓ URL updated: {}", input_buffer));
                }
            }
            EditField::Port => {
                if let Ok(port) = input_buffer.parse::<u16>() {
                    conn.port = port;
                    app.set_error(format!("✓ Port updated: {}", port));
                } else {
                    app.set_error(format!("✗ Invalid port: {} (must be a number)", input_buffer));
                }
            }
            EditField::Method => {
                let method = input_buffer.to_uppercase();
                if matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS") {
                    conn.method = method.clone();
                    app.set_error(format!("✓ Method updated: {}", method));
                } else {
                    app.set_error(format!("✗ Invalid method: {}", method));
                }
            }
        }
    }
}

fn handle_connecting_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') | KeyCode::Esc => {
            app.cancel_request();
        }
        _ => {}
    }
}

async fn send_request(app: &mut App, conn: &crate::models::Connection) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.request_cancel_tx = Some(tx);
    app.input_mode = InputMode::Connecting;
    
    let client = reqwest::Client::new();
    let url = conn.full_url();
    
    // Create a timeout future
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
    
    let result = tokio::select! {
        _ = rx => {
            app.input_mode = InputMode::Normal;
            return;
        }
        _ = timeout => {
            app.input_mode = InputMode::Normal;
            app.set_error("Request timeout (10 seconds)".to_string());
            return;
        }
        res = async {
            match conn.method.as_str() {
                "POST" | "PUT" | "PATCH" => {
                    let mut req = match conn.method.as_str() {
                        "POST" => client.post(&url),
                        "PUT" => client.put(&url),
                        "PATCH" => client.patch(&url),
                        _ => client.get(&url),
                    };

                    if let Some(payload) = &conn.payload {
                        req = req.body(payload.clone());
                    }

                    req.send().await
                }
                _ => client.get(&url).send().await,
            }
        } => res,
    };

    app.input_mode = InputMode::Normal;

    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            match resp.text().await {
                Ok(body) => {
                    app.response = Some(crate::models::ApiResponse {
                        status,
                        body,
                        headers: String::new(),
                    });
                }
                Err(e) => {
                    let error_msg = format!("Error reading response: {}", e);
                    app.response = Some(crate::models::ApiResponse {
                        status: 0,
                        body: error_msg,
                        headers: String::new(),
                    });
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Request failed: {}", e);
            app.response = Some(crate::models::ApiResponse {
                status: 0,
                body: error_msg,
                headers: String::new(),
            });
        }
    }
}
