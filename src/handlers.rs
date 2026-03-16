use crate::app::App;
use crate::models::{Connection, EditField, InputMode};
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
            if app.current_connection().is_some() {
                app.input_mode = InputMode::EditingConnection;
                app.edit_field = EditField::Name;
                app.input_buffer.clear();
                load_current_field_value(app);
                app.set_error("Editing connection - Tab to switch fields, Enter to confirm, Esc to cancel".to_string());
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
            app.select_prev();
        }
        KeyCode::Down => {
            app.select_next();
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
        KeyCode::PageDown => {
            app.scroll_response = app.scroll_response.saturating_add(5);
        }
        KeyCode::PageUp => {
            app.scroll_response = app.scroll_response.saturating_sub(5);
        }
        KeyCode::Home => {
            app.scroll_response = 0;
        }
        KeyCode::End => {
            app.scroll_response = u16::MAX;
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
                app.add_connection(conn);
                app.set_error(format!("✓ Connection '{}' created", name));
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
            apply_field_edit(app);
            app.input_buffer.clear();
            load_current_field_value(app);
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
            app.set_error("Editing cancelled".to_string());
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
                    app.set_error(format!("✓ Response received ({})", status));
                }
                Err(e) => {
                    app.set_error(format!("Error reading response: {}", e));
                }
            }
        }
        Err(e) => {
            app.set_error(format!("Request failed: {}", e));
        }
    }
}
