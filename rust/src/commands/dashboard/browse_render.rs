#![cfg(feature = "tui")]
use crate::tui_shell;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use super::browse_state::{BrowserState, PaneFocus, SearchDirection};
use super::delete_render::render_delete_dry_run_text;

#[path = "browse_render_detail.rs"]
mod browse_render_detail;
#[path = "browse_render_footer.rs"]
mod browse_render_footer;
#[path = "browse_render_rows.rs"]
mod browse_render_rows;

#[cfg(test)]
#[path = "browse_render_rust_tests.rs"]
mod browse_render_rust_tests;

use self::browse_render_detail::render_detail_panel;
use self::browse_render_footer::control_lines;
use self::browse_render_rows::build_tree_items;

pub(crate) fn render_dashboard_browser_frame(frame: &mut ratatui::Frame, state: &mut BrowserState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(4),
        ])
        .split(frame.area());
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(outer[1]);

    let header = tui_shell::build_header("Dashboard Browser", render_summary_lines(state));
    frame.render_widget(header, outer[0]);

    let list = List::new(build_tree_items(&state.document.nodes))
        .block(
            pane_block(
                "Tree",
                state.focus == PaneFocus::Tree,
                Color::LightBlue,
                Color::Rgb(14, 20, 27),
            )
            .title(format!(
                "Tree  {} org(s) / {} folder(s) / {} dashboard(s)",
                state.document.summary.org_count,
                state.document.summary.folder_count,
                state.document.summary.dashboard_count
            )),
        )
        .highlight_symbol("▌ ")
        .repeat_highlight_symbol(true)
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, panes[0], &mut state.list_state);

    render_detail_panel(frame, panes[1], state);

    let footer = tui_shell::build_footer(
        control_lines(
            state.pending_delete.is_some(),
            state.pending_edit.is_some(),
            state.pending_external_edit.is_some(),
            state.local_mode,
        ),
        state.status.clone(),
    );
    frame.render_widget(footer, outer[2]);

    if let Some(plan) = state.pending_delete.as_ref() {
        tui_shell::render_overlay(
            frame,
            "Delete Preview",
            render_delete_dry_run_text(plan)
                .into_iter()
                .map(Line::from)
                .collect(),
            Color::Red,
        );
    }
    if let Some(edit_state) = state.pending_edit.as_ref() {
        edit_state.render(frame);
    }
    if let Some(external_edit_state) = state.pending_external_edit.as_ref() {
        external_edit_state.render(frame);
    }
    if let Some(external_edit_error_state) = state.pending_external_edit_error.as_ref() {
        external_edit_error_state.render(frame);
    }
    if let Some(history_state) = state.pending_history.as_ref() {
        history_state.render(frame);
    }
    if let Some(search_state) = state.pending_search.as_ref() {
        render_search_prompt(frame, search_state.direction, &search_state.query);
    }
    if let Some(notice) = state.completion_notice.as_ref() {
        tui_shell::render_overlay(
            frame,
            &notice.title,
            vec![
                Line::from(notice.body.clone()),
                Line::from(""),
                Line::from("Press any key to continue."),
            ],
            Color::Green,
        );
    }
}

fn render_summary_lines(state: &BrowserState) -> Vec<Line<'static>> {
    let document = &state.document;
    vec![
        if document.summary.org_count > 1 {
            tui_shell::summary_line(&[
                tui_shell::summary_cell(
                    "Scope",
                    document.summary.scope_label.clone(),
                    Color::LightBlue,
                ),
                tui_shell::summary_cell(
                    "Orgs",
                    document.summary.org_count.to_string(),
                    Color::White,
                ),
                tui_shell::summary_cell(
                    "Folders",
                    document.summary.folder_count.to_string(),
                    Color::White,
                ),
                tui_shell::summary_cell(
                    "Dashboards",
                    document.summary.dashboard_count.to_string(),
                    Color::White,
                ),
                tui_shell::summary_cell(
                    "Root",
                    document
                        .summary
                        .root_path
                        .as_deref()
                        .unwrap_or("all folders"),
                    Color::White,
                ),
            ])
        } else {
            tui_shell::summary_line(&[
                tui_shell::summary_cell(
                    "Folders",
                    document.summary.folder_count.to_string(),
                    Color::White,
                ),
                tui_shell::summary_cell(
                    "Dashboards",
                    document.summary.dashboard_count.to_string(),
                    Color::White,
                ),
                tui_shell::summary_cell(
                    "Root",
                    document
                        .summary
                        .root_path
                        .as_deref()
                        .unwrap_or("all folders"),
                    Color::White,
                ),
            ])
        },
        if state.pending_delete.is_some() {
            Line::from(vec![
                tui_shell::label("Mode "),
                tui_shell::accent("confirm-delete", Color::LightRed),
                Span::raw("  "),
                tui_shell::focus_label("Focus "),
                tui_shell::key_chip(
                    match state.focus {
                        PaneFocus::Tree => "Tree",
                        PaneFocus::Facts => "Facts",
                    },
                    Color::Blue,
                ),
                Span::raw("  "),
                tui_shell::label("Confirm "),
                tui_shell::accent("y / Esc / q", Color::Yellow),
            ])
        } else {
            Line::from(vec![
                tui_shell::label("Mode "),
                tui_shell::accent(
                    if state.local_mode {
                        "local-browse"
                    } else {
                        "browse"
                    },
                    Color::Green,
                ),
                Span::raw("  "),
                tui_shell::focus_label("Focus "),
                tui_shell::key_chip(
                    match state.focus {
                        PaneFocus::Tree => "Tree",
                        PaneFocus::Facts => "Facts",
                    },
                    Color::Blue,
                ),
            ])
        },
    ]
}

fn pane_block(title: &str, focused: bool, accent: Color, bg: Color) -> Block<'static> {
    let title_bg = if focused { accent } else { bg };
    let title_fg = if focused { Color::Black } else { Color::White };
    Block::default()
        .borders(Borders::ALL)
        .title(if focused {
            format!("{title} [Focused]")
        } else {
            title.to_string()
        })
        .style(Style::default().bg(bg))
        .border_style(Style::default().fg(if focused { accent } else { Color::Gray }))
        .title_style(
            Style::default()
                .fg(title_fg)
                .bg(title_bg)
                .add_modifier(if focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )
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

fn render_search_prompt(frame: &mut ratatui::Frame, direction: SearchDirection, query: &str) {
    let area = ratatui::layout::Rect {
        x: frame.area().x + 6,
        y: frame.area().y + frame.area().height.saturating_sub(5),
        width: frame.area().width.saturating_sub(12).min(78),
        height: 3,
    };
    frame.render_widget(Clear, area);
    let prefix = match direction {
        SearchDirection::Forward => "/",
        SearchDirection::Backward => "?",
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
            "Enter search   Esc cancel   n repeat last search",
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
