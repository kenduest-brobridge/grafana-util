use crate::interactive_browser::{
    browser_detail_info_lines as build_info_lines, browser_review_empty_line,
    browser_review_info_lines,
};
use crate::tui_shell;
use crate::tui_shell::pane_block;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::super::datasource_browse_state::{BrowserState, PaneFocus};
use super::super::datasource_browse_support::{detail_lines, review_lines, DatasourceBrowseItem};
use super::render_summary::blank_dash;

pub(super) fn detail_title(state: &BrowserState) -> String {
    if let Some(pending_delete) = state.pending_delete.as_ref() {
        format!("Delete {} ({})", pending_delete.name, pending_delete.uid)
    } else {
        "Detail".to_string()
    }
}

pub(super) fn detail_text(state: &BrowserState) -> String {
    if let Some(pending_delete) = state.pending_delete.as_ref() {
        return [
            format!("Delete datasource {}", blank_dash(&pending_delete.name)),
            format!("UID: {}", blank_dash(&pending_delete.uid)),
            format!("ID: {}", pending_delete.id),
            String::new(),
            "Confirm: y".to_string(),
            "Cancel: n/Esc/q".to_string(),
        ]
        .join("\n");
    }
    state
        .selected_item()
        .map(|item| {
            if item.is_org_row() {
                return [
                    format!("Org: {}", blank_dash(&item.org)),
                    format!("Org ID: {}", blank_dash(&item.org_id)),
                    format!("Datasources: {}", item.datasource_count),
                    String::new(),
                    "Org rows are scope headers for all-org browsing.".to_string(),
                    "Select a datasource row to edit or delete.".to_string(),
                ]
                .join("\n");
            }
            let mut lines = vec![
                format!("Name: {}", blank_dash(&item.name)),
                format!("Type: {}", blank_dash(&item.datasource_type)),
                format!("UID: {}", blank_dash(&item.uid)),
                format!(
                    "Org: {} ({})",
                    blank_dash(&item.org),
                    blank_dash(&item.org_id)
                ),
                String::new(),
            ];
            lines.extend(detail_lines(item));
            lines.join("\n")
        })
        .unwrap_or_else(|| "No datasource selected.".to_string())
}

pub(super) fn render_detail_panel(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &BrowserState,
) {
    let Some(item) = state.selected_item() else {
        let empty = Paragraph::new("No datasource selected.")
            .block(Block::default().borders(Borders::ALL).title("Detail"));
        frame.render_widget(empty, area);
        return;
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(7),
            Constraint::Length(5),
            Constraint::Length(4),
        ])
        .split(area);

    let hero_lines = if item.is_org_row() {
        vec![
            Line::from(vec![
                Span::styled(
                    " ORG ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(53, 79, 122))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    item.org.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                format!(
                    "Org {}   {} datasource(s)",
                    blank_dash(&item.org_id),
                    item.datasource_count
                ),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(vec![
                tui_shell::muted("SCOPE "),
                tui_shell::boxed("all-org browse header", Color::Rgb(40, 49, 61)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    if item.is_default {
                        " DEFAULT "
                    } else {
                        " DATASOURCE "
                    },
                    Style::default()
                        .fg(Color::White)
                        .bg(if item.is_default {
                            Color::Rgb(18, 110, 52)
                        } else {
                            Color::Rgb(16, 92, 122)
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    item.name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                format!(
                    "{}   {}",
                    blank_dash(&item.datasource_type),
                    blank_dash(&item.uid)
                ),
                Style::default().fg(Color::Cyan),
            )),
            Line::from(vec![
                tui_shell::muted("URL "),
                tui_shell::boxed(blank_dash(&item.url), Color::Rgb(40, 49, 61)),
                Span::raw("   "),
                tui_shell::muted("ORG "),
                tui_shell::plain(format!(
                    "{} ({})",
                    blank_dash(&item.org),
                    blank_dash(&item.org_id)
                )),
            ]),
        ]
    };
    render_focusable_lines(
        frame,
        sections[0],
        hero_lines,
        pane_block("Overview", false, Color::LightBlue, Color::Rgb(18, 24, 33)).title("Overview"),
        false,
        state.detail_scroll,
    );

    render_focusable_lines(
        frame,
        sections[1],
        build_info_lines(&detail_lines(item)),
        pane_block(
            "Facts",
            state.focus == PaneFocus::Facts,
            Color::LightCyan,
            Color::Rgb(16, 20, 27),
        ),
        state.focus == PaneFocus::Facts,
        state.detail_scroll,
    );

    render_focusable_lines(
        frame,
        sections[2],
        datasource_review_panel_lines(item),
        pane_block(
            "Review",
            state.focus == PaneFocus::Review,
            Color::Yellow,
            Color::Rgb(28, 24, 16),
        ),
        state.focus == PaneFocus::Review,
        state.detail_scroll,
    );

    let shortcut_lines = if item.is_org_row() {
        vec![
            Line::from(vec![
                tui_shell::key_chip("Up/Down", Color::Blue),
                tui_shell::plain(" select org or datasource row"),
            ]),
            Line::from(vec![
                tui_shell::key_chip("l", Color::Cyan),
                tui_shell::plain(" refresh all visible orgs"),
                Span::raw("   "),
                tui_shell::key_chip("e/d", Color::DarkGray),
                tui_shell::plain(" datasource rows only"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                tui_shell::key_chip("e", Color::Green),
                tui_shell::plain(" edit datasource"),
            ]),
            Line::from(vec![
                tui_shell::key_chip("d", Color::Red),
                tui_shell::plain(" delete datasource"),
                Span::raw("   "),
                tui_shell::key_chip("l", Color::Cyan),
                tui_shell::plain(" refresh live data"),
            ]),
        ]
    };
    render_focusable_lines(
        frame,
        sections[3],
        shortcut_lines,
        pane_block(
            "Actions",
            false,
            Color::LightMagenta,
            Color::Rgb(22, 18, 30),
        )
        .title("Actions"),
        false,
        state.detail_scroll,
    );
}

pub(super) fn datasource_review_panel_lines(item: &DatasourceBrowseItem) -> Vec<Line<'static>> {
    if item.is_org_row() {
        return vec![browser_review_empty_line(
            "Select a datasource row to inspect review evidence.",
        )];
    }
    let lines = review_lines(item);
    if lines.is_empty() {
        return vec![browser_review_empty_line(
            "No secret placeholder or review-required evidence.",
        )];
    }
    browser_review_info_lines(&lines)
}

fn render_focusable_lines(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    lines: Vec<Line<'static>>,
    block: Block<'static>,
    focused: bool,
    scroll: u16,
) {
    let lines = if lines.is_empty() {
        vec![Line::from("-")]
    } else {
        lines
    };
    let items = lines.into_iter().map(ListItem::new).collect::<Vec<_>>();
    if focused {
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some((scroll as usize).min(items.len().saturating_sub(1))));
        let list = List::new(items)
            .block(block)
            .highlight_symbol("▌ ")
            .repeat_highlight_symbol(true)
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(list, area, &mut state);
    } else {
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
