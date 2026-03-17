use crate::app::App;
use crate::models::ActivePanel;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

fn draw_opaque_overlay(f: &mut Frame, area: Rect) {
    // Create a paragraph filled with spaces to create an opaque background
    let mut lines = Vec::new();
    for _ in 0..area.height {
        let spaces = " ".repeat(area.width as usize);
        lines.push(Line::from(Span::styled(
            spaces,
            Style::default().bg(Color::Black),
        )));
    }

    let overlay = Paragraph::new(lines).style(Style::default().bg(Color::Black));
    f.render_widget(overlay, area);
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(85), Constraint::Percentage(15)].as_ref())
        .split(f.area());

    // Top section (85%): connections on left, response on right
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[0]);

    // Draw connections list
    draw_connections(f, app, top_chunks[0]);

    // Draw response on the right
    draw_response(f, app, top_chunks[1]);

    // Draw help/shortcuts at the bottom
    draw_help(f, chunks[1]);

    // Draw modal dialogs (these overlay on top of everything)
    match app.input_mode {
        crate::models::InputMode::ConnectionName => {
            // Draw opaque overlay covering entire screen
            draw_opaque_overlay(f, f.area());
            let modal_area = fixed_rect(50, 10, f.area());
            draw_connection_name_dialog(f, app, modal_area);
        }
        crate::models::InputMode::EditingConnection => {
            // Draw opaque overlay covering entire screen
            draw_opaque_overlay(f, f.area());
            let modal_area = centered_rect(60, 60, f.area());
            draw_edit_dialog(f, app, modal_area);
        }
        crate::models::InputMode::Connecting => {
            // Draw opaque overlay covering entire screen
            draw_opaque_overlay(f, f.area());
            let modal_area = centered_rect(40, 10, f.area());
            draw_connecting_dialog(f, app, modal_area);
        }
        crate::models::InputMode::Normal => {
            // No modal in normal mode
        }
    }

    // Draw error message if present (overlay at bottom)
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
            y: f.area().height.saturating_sub(3),
            width: 50,
            height: 3,
        };

        f.render_widget(error_block, error_area);
    }
}

/// Calculate a centered rectangle with given fixed width and height in characters
fn fixed_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

/// Calculate a centered rectangle with given width and height percentages
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn draw_connections(f: &mut Frame, app: &App, area: Rect) {
    let connections: Vec<ListItem> = app
        .connections
        .iter()
        .map(|conn| ListItem::new(conn.name.as_str()))
        .collect();

    // Show visual indicator if this panel is active
    let title = if app.active_panel == ActivePanel::Connections {
        "◆ Connections ◆"
    } else {
        "Connections"
    };

    let list = List::new(connections)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(if app.active_panel == ActivePanel::Connections {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
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

fn draw_connection_name_dialog(f: &mut Frame, app: &App, area: Rect) {
    // The opaque overlay is already drawn by the caller; just render the dialog.

    // Build the input line: pad to fill the inner width, append a block cursor
    let inner_width = area.width.saturating_sub(4) as usize; // 2 border + 2 padding
    let input_with_cursor = format!("{}_", app.input_buffer);
    let padded = format!(
        "{:<width$}",
        input_with_cursor,
        width = inner_width.max(input_with_cursor.len())
    );

    let text = vec![
        Line::from(Span::styled(
            "Connection Name:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            padded,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" create  |  ", Style::default().fg(Color::White)),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" cancel", Style::default().fg(Color::White)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" New Connection ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Green))
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn draw_connecting_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Draw opaque background
    let background = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(background, area);

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
            Line::from(Span::styled(
                format!("{}:{}", conn.url, conn.port),
                Style::default().fg(Color::White),
            ))
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(paragraph, area);
}

fn draw_edit_dialog(f: &mut Frame, app: &App, area: Rect) {
    if let Some(conn) = app.current_connection() {
        // Draw opaque background with borders
        let background = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Black));
        f.render_widget(background, area);

        let mut text = vec![
            Line::from(Span::styled(
                "─ Edit Connection ─",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("", Style::default().bg(Color::Black))),
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

        text.push(Line::from(Span::styled(
            "",
            Style::default().bg(Color::Black),
        )));
        text.push(Line::from(vec![
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" save & next field  |  ", Style::default().bg(Color::Black)),
            Span::styled(
                "Shift+Tab",
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" save & prev", Style::default().bg(Color::Black)),
        ]));
        text.push(Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" save & exit  |  ", Style::default().bg(Color::Black)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Red)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" discard", Style::default().bg(Color::Black)),
        ]));

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Edit Connection")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow).bg(Color::Black)),
            )
            .style(Style::default().bg(Color::Black));

        f.render_widget(paragraph, area);
    }
}

fn draw_edit_field(
    label: &str,
    current_value: &str,
    input_buffer: &str,
    is_active: bool,
) -> Line<'static> {
    let label_text = format!("{:<10}", format!("{}:", label));

    if is_active {
        // Show input buffer with a block cursor so it's always visible
        let input_with_cursor = format!("{}_", input_buffer);
        let value_text = format!("[{}]", input_with_cursor);
        Line::from(vec![
            Span::styled(
                label_text,
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                value_text,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        let value_text = format!("[{}]", current_value);
        Line::from(Span::styled(
            format!("{}{}", label_text, value_text),
            Style::default().fg(Color::DarkGray).bg(Color::Black),
        ))
    }
}

fn draw_response(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(response) = &app.response {
        let mut lines = vec![];

        // Show connection info at the top
        if let Some(conn) = app.current_connection() {
            lines.push(Line::from(vec![
                Span::styled(
                    "Connection: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(conn.name.clone()),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "URL: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}:{}", conn.url, conn.port)),
            ]));
            lines.push(Line::from(""));
        }

        // Show error indicator if status is 0 (error)
        if response.status == 0 {
            lines.push(Line::from(Span::styled(
                "❌ ERROR",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    "Status: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(response.status.to_string()),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Body:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        }

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
        .block(
            Block::default()
                .title(if app.active_panel == ActivePanel::Response {
                    "◆ Response ◆ (j/k to scroll, Tab to switch)"
                } else {
                    "Response (j/k to scroll, Tab to switch)"
                })
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(if app.active_panel == ActivePanel::Response {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                }),
        )
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
                "Main: ",
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
            Span::raw("p/Tab"),
            Span::styled("-switch panel ", Style::default().fg(Color::DarkGray)),
            Span::raw("↑↓"),
            Span::styled("-navigate ", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                "Scroll: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("j/k"),
            Span::styled("-vim scroll ", Style::default().fg(Color::DarkGray)),
            Span::raw("PgUp/PgDn"),
            Span::styled("-page ", Style::default().fg(Color::DarkGray)),
            Span::raw("Home/End"),
            Span::styled("-jump ", Style::default().fg(Color::DarkGray)),
            Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Edit: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Tab"),
            Span::styled("-field ", Style::default().fg(Color::DarkGray)),
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
