use ratatui::{
    prelude::*,
    widgets::*,
};
use crate::{App, AppMode};

/// The main entry point for rendering. 
/// It decides which screen to show based on the current AppMode.
pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();

    match app.mode {
        AppMode::Setup => render_setup(f, app, size),
        AppMode::Login => render_login(f, app, size),
        AppMode::Main => render_main(f, app, size),
    }
}

/// Helper: Returns a glowing style if a field is currently being typed in.
fn get_input_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .bg(Color::Reset)
    }
}

/// Helper: Creates a centered rectangle for popups/modals.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Screen 1: First-time setup for Master Password and GitHub Config.
fn render_setup(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(70, 60, size);
    let block = Block::default()
        .title(" FERROCRYPT: INITIAL SETUP ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Green));
    
    let text = vec![
        Line::from("Welcome! Configure your vault and cloud settings:"),
        Line::from(""),
        Line::from(vec![
            Span::raw(" 1. Master Password:  "),
            Span::styled(format!(" {} ", app.master_pass_input), get_input_style(app.active_field == 0)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" 2. GitHub Token:     "),
            Span::styled(format!(" {} ", app.config.github_token), get_input_style(app.active_field == 1)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw(" 3. Gist ID:          "),
            Span::styled(format!(" {} ", app.config.gist_id), get_input_style(app.active_field == 2)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(" [TAB] Switch Fields  |  [ENTER] Save & Start ", Style::default().fg(Color::DarkGray))),
    ];

    f.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Left), area);
}

/// Screen 2: Login screen for returning users.
fn render_login(f: &mut Frame, app: &App, size: Rect) {
    let area = centered_rect(50, 30, size);
    let block = Block::default()
        .title(" UNLOCK VAULT ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let text = vec![
        Line::from("Enter Master Password to Decrypt:"),
        Line::from(""),
        // Password masking for security
        Line::from(Span::styled(
            format!(" > {} ", "*".repeat(app.master_pass_input.len())), 
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        )),
        Line::from(""),
        Line::from(Span::styled(" Press ENTER to Unlock ", Style::default().fg(Color::DarkGray))),
    ];

    f.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center), area);
}

/// Screen 3: The main dashboard (List + Details/Edit).
fn render_main(f: &mut Frame, app: &App, size: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Body
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Title Bar
    f.render_widget(
        Paragraph::new(" FerroCrypt Credential Vault ")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))),
        chunks[0]
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);
    
    // Left: Sidebar List
    let items: Vec<ListItem> = app.vault.entries.iter()
        .map(|e| ListItem::new(format!("  {}", e.title)))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(" Accounts ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
        .highlight_symbol(">> ");
    
    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(list, body[0], &mut state);

    // Right: Info or Edit Form
    if app.editing {
        let form = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  Title:    "), 
                Span::styled(format!(" {} ", app.title_input), get_input_style(app.active_field == 0))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Username: "), 
                Span::styled(format!(" {} ", app.user_input), get_input_style(app.active_field == 1))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Password: "), 
                Span::styled(format!(" {} ", app.pass_input), get_input_style(app.active_field == 2))
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  [TAB] Switch  |  [ENTER] Save  |  [ESC] Cancel ", Style::default().fg(Color::DarkGray))),
        ];
        f.render_widget(
            Paragraph::new(form)
                .block(Block::default()
                    .title(" ADD/EDIT ENTRY ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))),
            body[1]
        );
    } else if let Some(e) = app.vault.entries.get(app.selected_index) {
        let details = format!(
            "\n  ACCOUNT:  {}\n\n  USERNAME: {}\n\n  PASSWORD: {}", 
            e.title, e.username, e.password
        );
        f.render_widget(
            Paragraph::new(details)
                .block(Block::default().title(" INFO ").borders(Borders::ALL)),
            body[1]
        );
    } else {
        f.render_widget(
            Paragraph::new("\n  No entries found. Press 'A' to add.")
                .block(Block::default().title(" INFO ").borders(Borders::ALL)),
            body[1]
        );
    }

    // Footer
    let footer_msg = if app.editing {
        " Typing Mode... [ENTER] to save changes "
    } else {
        " [Q] Quit | [A] Add | [E] Edit | [S] Sync Gist"
    };

    f.render_widget(
        Paragraph::new(footer_msg)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL)),
        chunks[2]
    );
}
