#![cfg(feature = "tui")]
use crate::tui_shell;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::super::browse_state::{BrowserState, PaneFocus};
use super::super::browse_support::{DashboardBrowseNode, DashboardBrowseNodeKind};

pub(crate) fn render_detail_panel(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    state: &BrowserState,
) {
    let Some(node) = state.selected_node() else {
        let empty = ratatui::widgets::Paragraph::new("No item selected.").block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Detail"),
        );
        frame.render_widget(empty, area);
        return;
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(4),
        ])
        .split(area);

    let kind_color = match node.kind {
        DashboardBrowseNodeKind::Org => Color::Rgb(53, 79, 122),
        DashboardBrowseNodeKind::Folder => Color::Rgb(16, 92, 122),
        DashboardBrowseNodeKind::Dashboard => Color::Rgb(110, 78, 22),
    };
    let kind_label = match node.kind {
        DashboardBrowseNodeKind::Org => " ORG ",
        DashboardBrowseNodeKind::Folder => " FOLDER ",
        DashboardBrowseNodeKind::Dashboard => " DASHBOARD ",
    };
    let hero_lines = vec![
        Line::from(vec![
            Span::styled(
                kind_label,
                Style::default()
                    .fg(Color::White)
                    .bg(kind_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                node.title.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            match node.kind {
                DashboardBrowseNodeKind::Org => format!("Org {} ({})", node.org_name, node.org_id),
                _ => node.path.clone(),
            },
            Style::default().fg(Color::Cyan),
        )),
        Line::from(vec![
            muted("UID "),
            tui_shell::plain(
                node.uid
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .unwrap_or("-"),
            ),
            Span::raw("   "),
            muted("META "),
            plain_boxed(&node.meta, Color::Rgb(40, 49, 61)),
        ]),
    ];
    super::render_focusable_lines(
        frame,
        sections[0],
        hero_lines,
        super::pane_block("Overview", false, Color::LightBlue, Color::Rgb(18, 24, 33)),
        false,
        state.detail_scroll,
    );

    let detail_lines = detail_lines_for_node(node, &state.live_view_cache);
    super::render_focusable_lines(
        frame,
        sections[1],
        build_info_lines(&detail_lines),
        super::pane_block(
            "Facts",
            state.focus == PaneFocus::Facts,
            Color::LightCyan,
            Color::Rgb(16, 20, 27),
        ),
        state.focus == PaneFocus::Facts,
        state.detail_scroll,
    );

    super::render_focusable_lines(
        frame,
        sections[2],
        detail_shortcut_lines(node, state.local_mode),
        super::pane_block(
            "Actions",
            false,
            Color::LightMagenta,
            Color::Rgb(22, 18, 30),
        ),
        false,
        state.detail_scroll,
    );
}

fn build_info_lines(lines: &[String]) -> Vec<Line<'static>> {
    lines
        .iter()
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("Delete:"))
        .filter(|line| !line.starts_with("Delete folders:"))
        .filter(|line| !line.starts_with("Advanced edit:"))
        .filter(|line| !line.starts_with("View:"))
        .map(|line| {
            if line == "Live details:" {
                Line::from(vec![Span::styled(
                    "LIVE DETAILS",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                )])
            } else if let Some((label, value)) = line.split_once(':') {
                Line::from(vec![
                    Span::styled(
                        format!("{label:<18}: "),
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(value.trim().to_string(), Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect()
}

fn detail_shortcut_lines(node: &DashboardBrowseNode, local_mode: bool) -> Vec<Line<'static>> {
    match node.kind {
        DashboardBrowseNodeKind::Org => vec![
            Line::from(vec![
                tui_shell::key_chip("Up/Down", Color::Rgb(24, 78, 140)),
                tui_shell::plain(" select org, folder, or dashboard"),
            ]),
            Line::from(vec![
                tui_shell::key_chip("l", Color::Rgb(24, 78, 140)),
                tui_shell::plain(" refresh"),
                tui_shell::plain("   "),
                tui_shell::key_chip("/ ?", Color::Rgb(164, 116, 19)),
                tui_shell::plain(" search"),
                tui_shell::plain("   "),
                if local_mode {
                    tui_shell::key_chip("local", Color::Rgb(90, 98, 107))
                } else {
                    tui_shell::key_chip("e/d", Color::Rgb(90, 98, 107))
                },
                tui_shell::plain(if local_mode {
                    " read-only tree"
                } else {
                    " dashboard/folder rows only"
                }),
            ]),
        ],
        DashboardBrowseNodeKind::Folder => vec![
            Line::from(vec![
                tui_shell::key_chip("d", Color::Rgb(150, 38, 46)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " delete dashboards in subtree"
                }),
            ]),
            Line::from(vec![
                tui_shell::key_chip("D", Color::Rgb(150, 38, 46)),
                tui_shell::plain(if local_mode {
                    " live delete actions unavailable"
                } else {
                    " delete subtree + folders"
                }),
            ]),
        ],
        DashboardBrowseNodeKind::Dashboard => vec![
            Line::from(vec![
                tui_shell::key_chip("r", Color::Rgb(24, 106, 59)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " rename"
                }),
                tui_shell::plain("   "),
                tui_shell::key_chip("h", Color::Rgb(71, 55, 152)),
                tui_shell::plain(if local_mode {
                    " local history unavailable"
                } else {
                    " history"
                }),
                tui_shell::plain("   "),
                tui_shell::key_chip("m", Color::Rgb(24, 78, 140)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " move folder"
                }),
            ]),
            Line::from(vec![
                tui_shell::key_chip("e", Color::Rgb(71, 55, 152)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " metadata edit dialog"
                }),
                tui_shell::plain("   "),
                tui_shell::key_chip("E", Color::Rgb(71, 55, 152)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " raw JSON -> review/apply/save"
                }),
                tui_shell::plain("   "),
                tui_shell::key_chip("d", Color::Rgb(150, 38, 46)),
                tui_shell::plain(if local_mode {
                    " local browse is read-only"
                } else {
                    " delete"
                }),
            ]),
        ],
    }
}

fn detail_lines_for_node(
    node: &DashboardBrowseNode,
    live_view_cache: &std::collections::BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    if let Some(uid) = node.uid.as_ref() {
        if let Some(lines) = live_view_cache.get(&format!("{}::{uid}", node.org_id)) {
            return lines.clone();
        }
    }
    node.details.clone()
}

fn muted(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(Color::Gray))
}

fn plain_boxed(text: &str, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {} ", text),
        Style::default().fg(Color::White).bg(bg),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_node(kind: DashboardBrowseNodeKind) -> DashboardBrowseNode {
        DashboardBrowseNode {
            kind,
            title: "CPU Main".to_string(),
            path: "team/app".to_string(),
            uid: Some("abc123".to_string()),
            depth: 1,
            meta: "meta".to_string(),
            details: vec!["local detail".to_string(), "View: preview".to_string()],
            url: None,
            org_name: "Main Org.".to_string(),
            org_id: "1".to_string(),
            child_count: 0,
        }
    }

    #[test]
    fn detail_lines_for_node_prefers_live_cache_over_document_details() {
        let mut live_view_cache = BTreeMap::new();
        live_view_cache.insert(
            "1::abc123".to_string(),
            vec!["Live details:".to_string(), "folder: ops".to_string()],
        );
        let node = sample_node(DashboardBrowseNodeKind::Dashboard);
        let lines = detail_lines_for_node(&node, &live_view_cache);
        assert_eq!(
            lines,
            vec!["Live details:".to_string(), "folder: ops".to_string()]
        );
    }

    #[test]
    fn detail_shortcut_lines_surface_read_only_local_state() {
        let node = sample_node(DashboardBrowseNodeKind::Dashboard);
        let lines = detail_shortcut_lines(&node, true)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(lines
            .iter()
            .any(|line| line.contains("local browse is read-only")));
        assert!(lines
            .iter()
            .any(|line| line.contains("local history unavailable")));
    }
}
