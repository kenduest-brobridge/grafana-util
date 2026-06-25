use crate::tui_shell;
use ratatui::style::Color;
use ratatui::text::{Line, Span};

use super::super::datasource_browse_state::BrowserState;

pub(super) fn summary_lines(state: &BrowserState) -> Vec<Line<'static>> {
    let document = &state.document;
    vec![
        if document.org_count > 1 {
            tui_shell::summary_line(&[
                tui_shell::summary_cell(
                    "Scope",
                    blank_dash(&document.scope_label),
                    Color::LightBlue,
                ),
                tui_shell::summary_cell("Orgs", document.org_count.to_string(), Color::White),
                tui_shell::summary_cell(
                    "Datasources",
                    document.datasource_count.to_string(),
                    Color::White,
                ),
            ])
        } else {
            tui_shell::summary_line(&[
                tui_shell::summary_cell("Org", blank_dash(&document.org), Color::LightBlue),
                tui_shell::summary_cell("Id", blank_dash(&document.org_id), Color::White),
                tui_shell::summary_cell(
                    "Datasources",
                    document.datasource_count.to_string(),
                    Color::White,
                ),
            ])
        },
        Line::from(vec![
            tui_shell::label("Mode "),
            tui_shell::accent(
                if state.pending_delete.is_some() {
                    "confirm-delete"
                } else if state.pending_edit.is_some() {
                    "edit"
                } else if state.pending_search.is_some() {
                    "search"
                } else {
                    "browse"
                },
                if state.pending_delete.is_some() {
                    Color::LightRed
                } else if state.pending_edit.is_some() || state.pending_search.is_some() {
                    Color::Yellow
                } else {
                    Color::Green
                },
            ),
            Span::raw("  "),
            tui_shell::focus_label("Focus "),
            tui_shell::key_chip(state.focus_label(), Color::Blue),
            Span::raw("  "),
            tui_shell::label("Row "),
            tui_shell::accent(state.selected_position_summary(), Color::White),
            Span::raw("  "),
            tui_shell::label("Kind "),
            tui_shell::accent(state.selected_kind_summary(), Color::Yellow),
            Span::raw("  "),
            tui_shell::label("Search "),
            tui_shell::accent(state.search_summary(), Color::LightMagenta),
        ]),
    ]
}

pub(super) fn blank_dash(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "-"
    } else {
        trimmed
    }
}
