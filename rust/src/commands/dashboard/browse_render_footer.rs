#![cfg(feature = "tui")]
use crate::tui_shell;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub(crate) fn control_lines(
    has_pending_delete: bool,
    has_pending_edit: bool,
    has_pending_external_edit: bool,
    local_mode: bool,
) -> Vec<Line<'static>> {
    if local_mode && !has_pending_delete && !has_pending_edit && !has_pending_external_edit {
        return tui_shell::control_grid(&[
            vec![
                ("Up/Down", Color::Rgb(24, 78, 140), "move"),
                ("PgUp/PgDn", Color::Rgb(24, 78, 140), "scroll detail"),
                ("Tab", Color::Rgb(164, 116, 19), "next pane"),
                ("l", Color::Rgb(24, 78, 140), "refresh local tree"),
            ],
            vec![("/ ?", Color::Rgb(164, 116, 19), "search")],
        ])
        .into_iter()
        .chain(std::iter::once(Line::from(vec![Span::styled(
            "Local browse is read-only. Live edit, move, delete, and history actions are unavailable.",
            Style::default().fg(Color::Gray),
        )])))
        .collect();
    }
    if has_pending_delete {
        tui_shell::control_grid(&[
            vec![
                ("y", Color::Rgb(150, 38, 46), "confirm delete"),
                ("n", Color::Rgb(90, 98, 107), "cancel"),
                ("Esc", Color::Rgb(90, 98, 107), "cancel"),
                ("q", Color::Rgb(90, 98, 107), "cancel"),
            ],
            vec![("l", Color::Rgb(24, 78, 140), "refresh")],
        ])
    } else if has_pending_edit {
        tui_shell::control_grid(&[
            vec![
                ("Ctrl+S", Color::Rgb(24, 106, 59), "save"),
                ("Ctrl+X", Color::Rgb(90, 98, 107), "close"),
                ("Esc", Color::Rgb(90, 98, 107), "cancel"),
            ],
            vec![
                ("Tab", Color::Rgb(24, 78, 140), "next field"),
                ("Shift+Tab", Color::Rgb(24, 78, 140), "previous field"),
                ("Backspace", Color::Rgb(90, 98, 107), "delete char"),
            ],
        ])
    } else if has_pending_external_edit {
        tui_shell::control_grid(&[
            vec![
                ("a", Color::Rgb(24, 106, 59), "apply live"),
                ("w", Color::Rgb(164, 116, 19), "draft filename"),
                ("q", Color::Rgb(90, 98, 107), "discard"),
            ],
            vec![
                ("Enter", Color::Rgb(24, 106, 59), "apply live"),
                ("p", Color::Rgb(24, 78, 140), "refresh preview"),
            ],
        ])
    } else {
        tui_shell::control_grid(&[
            vec![
                ("Up/Down", Color::Rgb(24, 78, 140), "move"),
                ("PgUp/PgDn", Color::Rgb(24, 78, 140), "scroll detail"),
                ("Home/End", Color::Rgb(24, 78, 140), "jump"),
                ("Tab", Color::Rgb(164, 116, 19), "next pane"),
            ],
            vec![
                ("Shift+Tab", Color::Rgb(164, 116, 19), "previous pane"),
                ("/ ?", Color::Rgb(164, 116, 19), "search"),
                ("n", Color::Rgb(164, 116, 19), "next match"),
                ("r", Color::Rgb(24, 106, 59), "rename"),
                ("m", Color::Rgb(24, 78, 140), "move folder"),
            ],
            vec![
                ("d", Color::Rgb(150, 38, 46), "delete"),
                ("D", Color::Rgb(150, 38, 46), "delete+folders"),
                ("v", Color::Rgb(71, 55, 152), "live details"),
                ("h", Color::Rgb(71, 55, 152), "history"),
                ("e", Color::Rgb(71, 55, 152), "edit"),
                ("E", Color::Rgb(71, 55, 152), "raw json"),
                ("l", Color::Rgb(24, 78, 140), "refresh"),
                ("Esc/q", Color::Rgb(90, 98, 107), "exit"),
            ],
        ])
    }
}
