use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(65),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.area());

    // Top section: connections on left, request editor on right
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[0]);

    // Draw connections list
    draw_connections(f, app, top_chunks[0]);

    // Draw request editor
    draw_request_editor(f, app, top_chunks[1]);

    // Draw response in middle
    draw_response(f, app, chunks[1]);

    // Draw help/shortcuts at the bottom
    draw_help(f, chunks[2]);

    // Draw error message if present (overlay)
    if let Some(error) = &app.error_message {
        let error_text = vec![Line::from(vec![
            Span::styled("⚠ ", Style::default().fg(Color::Red)),
            Span::styled(error.clone(), Style::default().fg(Color::Red)),
        ])];

        let error_block = Paragraph::new(error_text).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray)),
        );

        let error_area = Rect {
            x: f.area().width.saturating_sub(50) / 2,
            y: f.area().height / 2,
            width: 50,
            height: 3,
        };

        f.render_widget(error_block, error_area);
    }
}

fn draw_connections(f: &mut Frame, app: &App, area: Rect) {
    let connections: Vec<ListItem> = app
        .connections
        .iter()
        .map(|conn| ListItem::new(conn.name.as_str()))
        .collect();

    let list = List::new(connections)
        .block(Block::default().title("Connections").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.selected_connection));

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_request_editor(f: &mut Frame, app: &App, area: Rect) {
    // Check if we're in edit mode or creating new connection
    match app.input_mode {
        crate::models::InputMode::EditingConnection => {
            draw_edit_dialog(f, app, area);
        }
        crate::models::InputMode::ConnectionName => {
            draw_connection_name_dialog(f, app, area);
        }
        crate::models::InputMode::Connecting => {
            draw_connecting_dialog(f, app, area);
        }
        crate::models::InputMode::Normal => {
            draw_connection_view(f, app, area);
        }
    }
}

fn draw_connection_view(f: &mut Frame, app: &App, area: Rect) {
    let mut text = if let Some(conn) = app.current_connection() {
        vec![
            Line::from(vec![
                Span::styled(
                    "Name: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&conn.name),
            ]),
            Line::from(vec![
                Span::styled("Method: ", Style::default().fg(Color::Cyan)),
                Span::raw(&conn.method),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Cyan)),
                Span::raw(&conn.url),
            ]),
            Line::from(vec![
                Span::styled("Port: ", Style::default().fg(Color::Cyan)),
                Span::raw(conn.port.to_string()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Query Parameters:",
                Style::default().fg(Color::Cyan),
            )),
        ]
    } else {
        vec![Line::from(
            "No connection selected - Press 'n' to create one",
        )]
    };

    // Add connection details if exists
    if let Some(conn) = app.current_connection() {
        for (key, value) in &conn.query_params {
            text.push(Line::from(format!("  {}: {}", key, value)));
        }

        text.push(Line::from(""));
        text.push(Line::from(Span::styled(
            "Payload:",
            Style::default().fg(Color::Cyan),
        )));

        if let Some(payload) = &conn.payload {
            text.push(Line::from(payload.as_str()));
        }
    }

    // Show help text
    text.push(Line::from(""));
    text.push(Line::from(vec![
        Span::styled("Press 'e' to edit | ", Style::default().fg(Color::DarkGray)),
        Span::styled("'r' to send | ", Style::default().fg(Color::DarkGray)),
        Span::styled("'s' to save", Style::default().fg(Color::DarkGray)),
    ]));

    let paragraph =
        Paragraph::new(text).block(Block::default().title("Request").borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn draw_connection_name_dialog(f: &mut Frame, app: &App, area: Rect) {
    let mut text = vec![
        Line::from(Span::styled(
            "─ Create New Connection ─",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw("Connection Name:")),
        Line::from(Span::styled(
            format!("> {}_", app.input_buffer),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" create  |  "),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn draw_connecting_dialog(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "─ Connecting... ─",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "⟳ Sending request...",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        if let Some(conn) = app.current_connection() {
            Line::from(format!("{}:{}", conn.url, conn.port))
        } else {
            Line::from("")
        },
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Ctrl+C",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn draw_edit_dialog(f: &mut Frame, app: &App, area: Rect) {
    if let Some(conn) = app.current_connection() {
        let mut text = vec![
            Line::from(Span::styled(
                "─ Edit Connection ─",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // Name field
        text.push(draw_edit_field(
            "Name",
            &conn.name,
            &app.input_buffer,
            matches!(app.edit_field, crate::models::EditField::Name),
        ));

        // URL field
        text.push(draw_edit_field(
            "URL",
            &conn.url,
            &app.input_buffer,
            matches!(app.edit_field, crate::models::EditField::Url),
        ));

        // Port field
        text.push(draw_edit_field(
            "Port",
            &conn.port.to_string(),
            &app.input_buffer,
            matches!(app.edit_field, crate::models::EditField::Port),
        ));

        // Method field
        text.push(draw_edit_field(
            "Method",
            &conn.method,
            &app.input_buffer,
            matches!(app.edit_field, crate::models::EditField::Method),
        ));

        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" next field  |  "),
            Span::styled(
                "Shift+Tab",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" prev field"),
        ]));
        text.push(Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" confirm  |  "),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel"),
        ]));

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .title("Edit Connection")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(paragraph, area);
    }
}

fn draw_edit_field(
    label: &str,
    current_value: &str,
    input_buffer: &str,
    is_active: bool,
) -> Line<'static> {
    let style = if is_active {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let value = if is_active {
        input_buffer.to_string()
    } else {
        current_value.to_string()
    };

    let label_text = format!("{:<10}", format!("{}:", label));
    let value_text = format!("[{}]", value);

    Line::from(Span::styled(format!("{}{}", label_text, value_text), style))
}

fn format_mode(mode: &crate::models::InputMode) -> String {
    match mode {
        crate::models::InputMode::Normal => "Normal".to_string(),
        crate::models::InputMode::ConnectionName => "Creating Connection Name".to_string(),
        crate::models::InputMode::EditingConnection => "Editing Connection".to_string(),
        crate::models::InputMode::Connecting => "Connecting...".to_string(),
    }
}

fn draw_response(f: &mut Frame, app: &App, area: Rect) {
    let mut text = if let Some(response) = &app.response {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "Status: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(response.status.to_string()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Body:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        // Try to format as JSON if it looks like JSON
        let body = if response.body.trim().starts_with('{') || response.body.trim().starts_with('[')
        {
            format_json(&response.body)
        } else {
            // Plain text - split into lines
            response.body.lines().map(|l| l.to_string()).collect()
        };

        // Add body lines with word wrapping
        for line in body {
            // Split long lines to fit the container width
            let available_width = area.width.saturating_sub(4) as usize; // Leave margin for borders
            if available_width > 10 {
                for wrapped_line in wrap_text(&line, available_width) {
                    lines.push(Line::from(wrapped_line));
                }
            } else {
                lines.push(Line::from(line));
            }
        }

        lines
    } else {
        vec![Line::from("No response yet")]
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Response").borders(Borders::ALL))
        .scroll((app.scroll_response, 0));

    f.render_widget(paragraph, area);
}

/// Format JSON with indentation for readability
fn format_json(json_str: &str) -> Vec<String> {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(value) => {
            // Pretty print with 2-space indentation
            match serde_json::to_string_pretty(&value) {
                Ok(formatted) => formatted.lines().map(|l| l.to_string()).collect(),
                Err(_) => json_str.lines().map(|l| l.to_string()).collect(),
            }
        }
        Err(_) => {
            // If parsing fails, just return the original lines
            json_str.lines().map(|l| l.to_string()).collect()
        }
    }
}

/// Wrap text to fit within a given width
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width < 10 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Line is full, start a new one
            if !current_line.is_empty() {
                result.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    if result.is_empty() {
        result.push(text.to_string());
    }

    result
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled(
                "Controls: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("n"),
            Span::styled("-new ", Style::default().fg(Color::DarkGray)),
            Span::raw("e"),
            Span::styled("-edit ", Style::default().fg(Color::DarkGray)),
            Span::raw("d"),
            Span::styled("-delete ", Style::default().fg(Color::DarkGray)),
            Span::raw("r"),
            Span::styled("-send ", Style::default().fg(Color::DarkGray)),
            Span::raw("s"),
            Span::styled("-save ", Style::default().fg(Color::DarkGray)),
            Span::raw("↑↓"),
            Span::styled("-navigate ", Style::default().fg(Color::DarkGray)),
            Span::raw("PgUp/PgDn"),
            Span::styled("-scroll response ", Style::default().fg(Color::DarkGray)),
            Span::raw("Ctrl+Q"),
            Span::styled("-quit", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                "Edit mode: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Tab"),
            Span::styled("-next field ", Style::default().fg(Color::DarkGray)),
            Span::raw("Shift+Tab"),
            Span::styled("-prev field ", Style::default().fg(Color::DarkGray)),
            Span::raw("Enter"),
            Span::styled("-confirm ", Style::default().fg(Color::DarkGray)),
            Span::raw("Esc"),
            Span::styled("-cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().title("Shortcuts").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}
