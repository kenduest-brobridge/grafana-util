#![cfg(feature = "tui")]

use crate::tui_shell;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub(super) fn control_lines(
    has_pending_delete: bool,
    has_pending_edit: bool,
) -> Vec<Line<'static>> {
    if has_pending_delete {
        return vec![tui_shell::fixed_body_control_line(
            &[
                ("y", Color::Red, "confirm delete"),
                ("n", Color::Gray, "cancel"),
                ("Esc", Color::Gray, "cancel"),
                ("q", Color::Gray, "cancel"),
            ],
            14,
        )];
    }
    if has_pending_edit {
        return vec![tui_shell::fixed_body_control_line(
            &[
                ("Ctrl+S", Color::Green, "save"),
                ("Esc", Color::Gray, "cancel"),
                ("Ctrl+X", Color::Gray, "close"),
                ("Tab", Color::Blue, "next field"),
                ("Shift+Tab", Color::Blue, "previous field"),
            ],
            14,
        )];
    }
    vec![
        tui_shell::fixed_body_control_line(
            &[
                ("Up/Down", Color::Blue, "move"),
                ("PgUp/PgDn", Color::Blue, "scroll detail"),
                ("Tab", Color::Blue, "next pane"),
                ("e", Color::Green, "edit"),
                ("d", Color::Red, "delete"),
            ],
            14,
        ),
        tui_shell::fixed_body_control_line(
            &[
                ("Shift+Tab", Color::Blue, "previous pane"),
                ("/ ?", Color::Yellow, "search"),
                ("n", Color::Yellow, "next match"),
                ("l", Color::Cyan, "refresh"),
                ("Home/End", Color::Blue, "jump"),
            ],
            14,
        ),
        tui_shell::fixed_body_control_line(&[("Esc/q", Color::Gray, "exit")], 14),
    ]
}

pub(super) fn render_search_prompt(
    frame: &mut ratatui::Frame,
    direction: super::datasource_browse_state::SearchDirection,
    query: &str,
) {
    let area = ratatui::layout::Rect {
        x: frame.area().x + 6,
        y: frame.area().y + frame.area().height.saturating_sub(6),
        width: frame.area().width.saturating_sub(12).min(70),
        height: 4,
    };
    frame.render_widget(Clear, area);
    let prefix = match direction {
        super::datasource_browse_state::SearchDirection::Forward => "/",
        super::datasource_browse_state::SearchDirection::Backward => "?",
    };
    let prompt = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", prefix),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(164, 116, 19))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(query.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(Span::styled(
            "Enter search   Esc cancel   n repeat",
            Style::default().fg(Color::Gray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .style(Style::default().bg(Color::Rgb(18, 20, 26)))
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .style(Style::default().bg(Color::Rgb(18, 20, 26)));
    frame.render_widget(prompt, area);
}
