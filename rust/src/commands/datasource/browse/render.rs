#![cfg(feature = "tui")]

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

use super::datasource_browse_render_chrome::{control_lines, render_search_prompt};
use super::datasource_browse_state::{BrowserState, PaneFocus};
use super::datasource_browse_support::{detail_lines, review_lines, DatasourceBrowseItem};

pub(crate) fn render_datasource_browser_frame(
    frame: &mut ratatui::Frame,
    state: &mut BrowserState,
) {
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
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(outer[1]);

    let header = tui_shell::build_header("Datasource Browser", summary_lines(state));
    frame.render_widget(header, outer[0]);

    let list = List::new(build_list_items(&state.document.items))
        .block(
            pane_block(
                "List",
                state.focus == PaneFocus::List,
                Color::LightBlue,
                Color::Rgb(14, 20, 27),
            )
            .title(format!(
                "List  {} org(s) / {} datasource(s)",
                state.document.org_count, state.document.datasource_count
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
        control_lines(state.pending_delete.is_some(), state.pending_edit.is_some()),
        state.status.clone(),
    );
    frame.render_widget(footer, outer[2]);

    if let Some(edit_state) = state.pending_edit.as_ref() {
        edit_state.render(frame);
    }
    if state.pending_delete.is_some() {
        tui_shell::render_overlay(
            frame,
            &detail_title(state),
            detail_text(state)
                .lines()
                .map(|line| Line::from(line.to_string()))
                .collect(),
            Color::Red,
        );
    }
    if let Some(search_state) = state.pending_search.as_ref() {
        render_search_prompt(frame, search_state.direction, &search_state.query);
    }
}

fn summary_lines(state: &BrowserState) -> Vec<Line<'static>> {
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

fn build_list_items(items: &[DatasourceBrowseItem]) -> Vec<ListItem<'_>> {
    let mut rendered = Vec::new();
    for (index, item) in items.iter().enumerate() {
        if item.is_org_row() {
            let line = Line::from(vec![
                Span::styled(
                    " ORG ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(46, 66, 98))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{} ", item.org),
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "│ id={} │ {} datasource(s)",
                        item.org_id, item.datasource_count
                    ),
                    Style::default().fg(Color::Gray),
                ),
            ]);
            if index > 0 {
                rendered.push(ListItem::new(vec![Line::from(Span::raw(" ")), line]));
            } else {
                rendered.push(ListItem::new(line));
            }
            continue;
        }
        let badge_color = if item.is_default {
            Color::Green
        } else {
            Color::DarkGray
        };
        let branch = datasource_tree_branch(items, index);
        let line = Line::from(vec![
            Span::styled("     ", Style::default().fg(Color::DarkGray)),
            Span::styled(branch, Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled(
                if item.is_default { "[*]" } else { "[ ]" },
                Style::default().fg(Color::White).bg(badge_color),
            ),
            Span::raw(" "),
            Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  │  {}  │  {}",
                    blank_dash(&item.datasource_type),
                    blank_dash(&item.uid)
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        rendered.push(ListItem::new(line));
    }
    rendered
}

fn datasource_tree_branch(items: &[DatasourceBrowseItem], index: usize) -> &'static str {
    let has_next_datasource_in_same_group = items
        .get(index + 1)
        .map(|next| !next.is_org_row())
        .unwrap_or(false);
    if has_next_datasource_in_same_group {
        "├─"
    } else {
        "└─"
    }
}

fn detail_title(state: &BrowserState) -> String {
    if let Some(pending_delete) = state.pending_delete.as_ref() {
        format!("Delete {} ({})", pending_delete.name, pending_delete.uid)
    } else {
        "Detail".to_string()
    }
}

fn detail_text(state: &BrowserState) -> String {
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

fn render_detail_panel(
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

fn datasource_review_panel_lines(item: &DatasourceBrowseItem) -> Vec<Line<'static>> {
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

fn blank_dash(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "-"
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::super::datasource_browse_support::DatasourceBrowseDocument;
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn datasource_browse_render_does_not_wrap_muted_shell_span() {
        let source = include_str!("render.rs");
        let wrapper_signature = format!("{}{}(", "fn ", "muted");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse rendering should call tui_shell::muted directly instead of \
             carrying a local muted delegate wrapper"
        );
    }

    #[test]
    fn datasource_browse_render_does_not_wrap_boxed_shell_span() {
        let source = include_str!("render.rs");
        let wrapper_signature = format!("{}{}(", "fn ", "plain_boxed");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse rendering should call tui_shell::boxed directly instead of \
             carrying a local plain_boxed delegate wrapper"
        );
    }

    #[test]
    fn datasource_browse_render_does_not_wrap_control_line_shell_rows() {
        let source = include_str!("render.rs");
        let wrapper_signature = format!("{}{}(", "fn ", "control_line");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse rendering should call shared tui_shell control-line helpers directly \
             instead of carrying a local control_line delegate wrapper"
        );
    }

    fn empty_document() -> DatasourceBrowseDocument {
        DatasourceBrowseDocument {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            org_count: 1,
            datasource_count: 0,
            scope_label: "current-org".to_string(),
            items: Vec::new(),
        }
    }

    #[test]
    fn summary_lines_surface_focus_and_mode() {
        let state = BrowserState::new(empty_document());
        let lines = summary_lines(&state)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("Mode"));
        assert!(lines[1].contains("browse"));
        assert!(lines[1].contains("Focus"));
        assert!(lines[1].contains("list"));
        assert!(lines[1].contains("Row"));
        assert!(lines[1].contains("-"));
        assert!(lines[1].contains("Kind"));
        assert!(lines[1].contains("none"));
        assert!(lines[1].contains("Search"));
        assert!(!lines.iter().any(|line| line.contains("default datasource")));
    }

    #[test]
    fn summary_lines_surface_pending_delete_mode() {
        let mut state = BrowserState::new(empty_document());
        state.pending_delete = Some(super::super::datasource_browse_state::PendingDelete {
            uid: "uid-1".to_string(),
            name: "Prom".to_string(),
            id: 7,
        });
        let lines = summary_lines(&state)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("Mode"));
        assert!(lines[1].contains("confirm-delete"));
        assert!(lines[1].contains("Focus"));
        assert!(lines[1].contains("list"));
    }

    #[test]
    fn pending_delete_detail_uses_compact_confirmation_controls() {
        let mut state = BrowserState::new(empty_document());
        state.pending_delete = Some(super::super::datasource_browse_state::PendingDelete {
            uid: "uid-1".to_string(),
            name: "Prom".to_string(),
            id: 7,
        });

        let rendered = detail_text(&state);

        assert!(rendered.contains("Confirm: y"));
        assert!(rendered.contains("Cancel: n/Esc/q"));
        assert!(!rendered.contains("Press n, Esc, or q"));
    }

    #[test]
    fn search_prompt_uses_compact_apply_cancel_repeat_hint() {
        let mut terminal = Terminal::new(TestBackend::new(90, 16)).unwrap();

        terminal
            .draw(|frame| {
                render_search_prompt(
                    frame,
                    super::super::datasource_browse_state::SearchDirection::Backward,
                    "prom",
                )
            })
            .unwrap();

        let screen = format!("{}", terminal.backend());
        assert!(screen.contains("Enter search"));
        assert!(screen.contains("Esc cancel"));
        assert!(screen.contains("n repeat"));
        assert!(!screen.contains("repeat last search"));
    }

    #[test]
    fn summary_lines_surface_selection_and_search_context() {
        let mut state = BrowserState::new(DatasourceBrowseDocument {
            org: "All visible orgs".to_string(),
            org_id: "-".to_string(),
            org_count: 2,
            datasource_count: 1,
            scope_label: "all-orgs".to_string(),
            items: vec![
                DatasourceBrowseItem {
                    kind: super::super::datasource_browse_support::DatasourceBrowseItemKind::Org,
                    depth: 0,
                    id: 0,
                    uid: String::new(),
                    name: "Main Org.".to_string(),
                    datasource_type: "org".to_string(),
                    access: String::new(),
                    url: String::new(),
                    is_default: false,
                    org: "Main Org.".to_string(),
                    org_id: "1".to_string(),
                    details: serde_json::Map::new(),
                    datasource_count: 1,
                },
                DatasourceBrowseItem {
                    kind: super::super::datasource_browse_support::DatasourceBrowseItemKind::Datasource,
                    depth: 1,
                    id: 9,
                    uid: "smoke-prom".to_string(),
                    name: "Smoke Prometheus".to_string(),
                    datasource_type: "prometheus".to_string(),
                    access: "proxy".to_string(),
                    url: "http://prom".to_string(),
                    is_default: false,
                    org: "Main Org.".to_string(),
                    org_id: "1".to_string(),
                    details: serde_json::Map::new(),
                    datasource_count: 0,
                },
            ],
        });
        state.select_last();
        state.last_search = Some(super::super::datasource_browse_state::SearchState {
            direction: super::super::datasource_browse_state::SearchDirection::Forward,
            query: "smoke".to_string(),
        });
        let lines = summary_lines(&state)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(lines[1].contains("Row"));
        assert!(lines[1].contains("2/2"));
        assert!(lines[1].contains("Kind"));
        assert!(lines[1].contains("datasource"));
        assert!(lines[1].contains("Search"));
        assert!(lines[1].contains("/smoke"));
    }

    #[test]
    fn control_lines_surface_consistent_focus_cycle_and_exit_labels() {
        let lines = control_lines(false, false)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(lines[0].contains("next pane"));
        assert!(lines[1].contains("previous pane"));
        assert!(lines[1].contains("search"));
        assert!(lines[2].contains("exit"));
        assert!(lines[2].contains("Esc/q"));
    }

    #[test]
    fn shared_browser_info_lines_format_datasource_detail_rows() {
        let lines = crate::interactive_browser::browser_detail_info_lines(&[
            "UID: smoke-prom".to_string(),
            String::new(),
            "No colon row".to_string(),
        ])
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("UID"));
        assert!(lines[0].contains("smoke-prom"));
        assert!(lines[1].contains("No colon row"));
    }

    #[test]
    fn review_lines_surface_secret_evidence_without_resolved_values() {
        let item = DatasourceBrowseItem {
            kind: super::super::datasource_browse_support::DatasourceBrowseItemKind::Datasource,
            depth: 1,
            id: 9,
            uid: "secure-prom".to_string(),
            name: "Secure Prometheus".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prom".to_string(),
            is_default: false,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            details: serde_json::json!({
                "secureJsonData": {
                    "password": "super-secret-value"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
            datasource_count: 0,
        };

        let rendered = datasource_review_panel_lines(&item)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Secret material"));
        assert!(rendered.contains("Secret review required"));
        assert!(rendered.contains("resolved credential values are never displayed"));
        assert!(!rendered.contains("super-secret-value"));
    }

    #[test]
    fn review_pane_formats_local_review_evidence_without_secret_values() {
        let item = DatasourceBrowseItem {
            kind: super::super::datasource_browse_support::DatasourceBrowseItemKind::Datasource,
            depth: 1,
            id: 9,
            uid: "secure-prom".to_string(),
            name: "Secure Prometheus".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prom".to_string(),
            is_default: false,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            details: serde_json::json!({
                "action": "would-update",
                "status": "ready",
                "matchBasis": "uid",
                "targetReadOnly": false,
                "changedFields": ["url", "jsonData"],
                "requiresSecretValues": true,
                "secureJsonData": {
                    "password": "super-secret-value"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
            datasource_count: 0,
        };

        let rendered = datasource_review_panel_lines(&item)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Review action"));
        assert!(rendered.contains("would-update"));
        assert!(rendered.contains("Review match"));
        assert!(rendered.contains("uid"));
        assert!(rendered.contains("Review changed fields"));
        assert!(rendered.contains("jsonData, url"));
        assert!(rendered.contains("Review requires secret values"));
        assert!(!rendered.contains("super-secret-value"));
    }

    #[test]
    fn shared_browser_review_lines_format_datasource_review_rows() {
        let lines = crate::interactive_browser::browser_review_info_lines(&[
            "Review action: would-update".to_string(),
            "Review required: true".to_string(),
            "plain review note".to_string(),
        ])
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Review action"));
        assert!(lines[0].contains("would-update"));
        assert!(lines[1].contains("Review required"));
        assert!(lines[1].contains("true"));
        assert!(lines[2].contains("plain review note"));
    }

    #[test]
    fn datasource_review_panel_does_not_keep_generic_build_review_wrapper() {
        let source = include_str!("render.rs");
        let wrapper_signature = format!("{}{}(", "fn ", "build_review_lines");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse review rendering should use a domain-specific panel builder name \
             instead of carrying a generic build_review_lines helper-drift candidate"
        );
    }
}
