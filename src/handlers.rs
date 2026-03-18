use crate::app::App;
use crate::models::{Connection, EditField, InputMode, ActivePanel, KeyValueTarget};
use crossterm::event::{KeyCode, KeyEvent};

pub async fn handle_input(app: &mut App, key: KeyEvent) {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key).await,
        InputMode::ConnectionName => handle_connection_name_input(app, key),
        InputMode::EditingConnection => handle_edit_dialog_input(app, key),
        InputMode::EditingPayload => handle_payload_editor_input(app, key),
        InputMode::EditingKeyValue => handle_keyvalue_editor_input(app, key),
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
                let backup = conn.clone();
                // Initialize method_index from current connection's method
                app.method_index = App::method_index_from_string(&backup.method);
                app.edit_backup = Some(backup);
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
    let is_method_field = matches!(app.edit_field, EditField::Method);
    let is_payload_field = matches!(app.edit_field, EditField::Payload);
    let is_headers_field = matches!(app.edit_field, EditField::Headers);
    let is_query_params_field = matches!(app.edit_field, EditField::QueryParams);
    let is_special_field = is_method_field || is_payload_field || is_headers_field || is_query_params_field;
    
    match key.code {
        KeyCode::Tab => {
            // Save the current field before moving to next
            apply_field_edit(app);
            // Move to next field
            app.edit_field = match app.edit_field {
                EditField::Name => EditField::Url,
                EditField::Url => EditField::Port,
                EditField::Port => EditField::Method,
                EditField::Method => EditField::Headers,
                EditField::Headers => EditField::QueryParams,
                EditField::QueryParams => EditField::Payload,
                EditField::Payload => EditField::Name,
            };
            app.input_buffer.clear();
            load_current_field_value(app);
        }
        KeyCode::BackTab => {
            // Save the current field before moving to previous
            apply_field_edit(app);
            // Move to previous field
            app.edit_field = match app.edit_field {
                EditField::Name => EditField::Payload,
                EditField::Url => EditField::Name,
                EditField::Port => EditField::Url,
                EditField::Method => EditField::Port,
                EditField::Headers => EditField::Method,
                EditField::QueryParams => EditField::Headers,
                EditField::Payload => EditField::QueryParams,
            };
            app.input_buffer.clear();
            load_current_field_value(app);
        }
        KeyCode::Left if is_method_field => {
            app.prev_method();
        }
        KeyCode::Right if is_method_field => {
            app.next_method();
        }
        KeyCode::Char(c) if !is_special_field => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace if !is_special_field => {
            app.input_buffer.pop();
        }
        KeyCode::Enter if is_payload_field => {
            // Open payload editor
            let payload = app.current_connection()
                .and_then(|c| c.payload.clone());
            app.init_payload_editor(payload.as_deref());
            app.input_mode = InputMode::EditingPayload;
        }
        KeyCode::Enter if is_headers_field => {
            // Open key-value editor for headers
            app.init_kv_editor(KeyValueTarget::Headers);
            app.input_mode = InputMode::EditingKeyValue;
        }
        KeyCode::Enter if is_query_params_field => {
            // Open key-value editor for query params
            app.init_kv_editor(KeyValueTarget::QueryParams);
            app.input_mode = InputMode::EditingKeyValue;
        }
        KeyCode::Enter => {
            // Save the current field and exit the dialog
            let old_name = app.edit_backup.as_ref().map(|b| b.name.clone());
            apply_field_edit(app);
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
            app.edit_backup = None;
            // Save to disk, and delete old file if name changed
            if let Some(conn) = app.current_connection() {
                let conn_clone = conn.clone();
                // Delete old file if the name changed
                if let Some(old) = old_name {
                    if old != conn_clone.name {
                        let _ = app.delete_connection_file(&old);
                    }
                }
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
                app.method_index = App::method_index_from_string(&conn.method);
            }
            EditField::Headers => {
                // Headers use the key-value editor
            }
            EditField::QueryParams => {
                // Query params use the key-value editor
            }
            EditField::Payload => {
                // Payload doesn't use input_buffer, it uses the payload editor
            }
        }
    }
}

fn apply_field_edit(app: &mut App) {
    let edit_field = app.edit_field.clone();
    let input_buffer = app.input_buffer.clone();
    let current_method = app.current_method().to_string();
    
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
                conn.method = current_method.clone();
                app.set_error(format!("✓ Method updated: {}", current_method));
            }
            EditField::Headers => {
                // Headers are edited in key-value editor mode
            }
            EditField::QueryParams => {
                // Query params are edited in key-value editor mode
            }
            EditField::Payload => {
                // Payload is edited in its own editor mode, not here
            }
        }
    }
}

fn handle_payload_editor_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            // Discard changes and go back to edit dialog
            app.input_mode = InputMode::EditingConnection;
            app.set_error("✗ Payload changes discarded".to_string());
        }
        KeyCode::F(2) => {
            // Save payload and go back to edit dialog
            let payload = app.payload_to_string();
            if let Some(conn) = app.current_connection_mut() {
                conn.payload = payload.clone();
            }
            app.input_mode = InputMode::EditingConnection;
            if payload.is_some() {
                app.set_error("✓ Payload saved".to_string());
            } else {
                app.set_error("✓ Payload cleared".to_string());
            }
        }
        KeyCode::Up => {
            app.payload_move_up();
        }
        KeyCode::Down => {
            app.payload_move_down();
        }
        KeyCode::Left => {
            app.payload_move_left();
        }
        KeyCode::Right => {
            app.payload_move_right();
        }
        KeyCode::Home => {
            app.payload_move_home();
        }
        KeyCode::End => {
            app.payload_move_end();
        }
        KeyCode::Enter => {
            app.payload_newline();
        }
        KeyCode::Backspace => {
            app.payload_backspace();
        }
        KeyCode::Delete => {
            app.payload_delete();
        }
        KeyCode::Tab => {
            // Insert 2 spaces for indentation
            app.payload_insert_char(' ');
            app.payload_insert_char(' ');
        }
        KeyCode::Char(c) => {
            app.payload_insert_char(c);
        }
        _ => {}
    }
}

fn handle_keyvalue_editor_input(app: &mut App, key: KeyEvent) {
    // Check if we're currently editing a key or value
    if app.kv_editing.is_some() {
        match key.code {
            KeyCode::Esc => {
                app.kv_cancel_edit();
            }
            KeyCode::Enter | KeyCode::Tab => {
                app.kv_save_edit();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            _ => {}
        }
        return;
    }
    
    // Not editing - handle navigation and commands
    match key.code {
        KeyCode::Esc => {
            // Discard changes and go back to edit dialog
            app.input_mode = InputMode::EditingConnection;
            let target_name = match app.kv_target {
                KeyValueTarget::Headers => "Headers",
                KeyValueTarget::QueryParams => "Query params",
            };
            app.set_error(format!("✗ {} changes discarded", target_name));
        }
        KeyCode::F(2) => {
            // Save and go back to edit dialog
            let kv_map = app.kv_to_hashmap();
            let target = app.kv_target.clone();
            if let Some(conn) = app.current_connection_mut() {
                match target {
                    KeyValueTarget::Headers => conn.headers = kv_map,
                    KeyValueTarget::QueryParams => conn.query_params = kv_map,
                }
            }
            app.input_mode = InputMode::EditingConnection;
            let target_name = match target {
                KeyValueTarget::Headers => "Headers",
                KeyValueTarget::QueryParams => "Query params",
            };
            app.set_error(format!("✓ {} saved", target_name));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.kv_move_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.kv_move_down();
        }
        KeyCode::Char('a') | KeyCode::Char('n') => {
            // Add new item
            app.kv_add_item();
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            // Delete selected item
            app.kv_delete_selected();
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            // Edit key of selected item
            if !app.kv_items.is_empty() {
                app.kv_start_edit_key();
            }
        }
        KeyCode::Char('v') => {
            // Edit value of selected item
            if !app.kv_items.is_empty() {
                app.kv_start_edit_value();
            }
        }
        KeyCode::Tab => {
            // Toggle between editing key and value
            if !app.kv_items.is_empty() {
                app.kv_start_edit_value();
            }
        }
        _ => {}
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
    
    // Clone headers for use in async block
    let custom_headers = conn.headers.clone();
    
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

                    // Add custom headers
                    for (key, value) in &custom_headers {
                        req = req.header(key, value);
                    }

                    if let Some(payload) = &conn.payload {
                        req = req.body(payload.clone());
                    }

                    req.send().await
                }
                "DELETE" => {
                    let mut req = client.delete(&url);
                    for (key, value) in &custom_headers {
                        req = req.header(key, value);
                    }
                    req.send().await
                }
                "HEAD" => {
                    let mut req = client.head(&url);
                    for (key, value) in &custom_headers {
                        req = req.header(key, value);
                    }
                    req.send().await
                }
                _ => {
                    // GET and OPTIONS
                    let mut req = client.get(&url);
                    for (key, value) in &custom_headers {
                        req = req.header(key, value);
                    }
                    req.send().await
                }
            }
        } => res,
    };

    app.input_mode = InputMode::Normal;

    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            // Capture headers before consuming response
            let headers = resp.headers()
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("<binary>")))
                .collect::<Vec<_>>()
                .join("\n");
            match resp.text().await {
                Ok(body) => {
                    app.response = Some(crate::models::ApiResponse {
                        status,
                        body,
                        headers,
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
