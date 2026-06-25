#![cfg(feature = "tui")]

use crate::tui_shell;
use crate::tui_shell::pane_block;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::List;

use super::datasource_browse_render_chrome::{control_lines, render_search_prompt};
use super::datasource_browse_state::{BrowserState, PaneFocus};

#[path = "render_detail.rs"]
mod render_detail;
#[path = "render_list.rs"]
mod render_list;
#[path = "render_summary.rs"]
mod render_summary;

#[cfg(test)]
use render_detail::datasource_review_panel_lines;
use render_detail::{detail_text, detail_title, render_detail_panel};
use render_list::build_list_items;
use render_summary::summary_lines;

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

#[cfg(test)]
mod tests {
    use super::super::datasource_browse_support::{DatasourceBrowseDocument, DatasourceBrowseItem};
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render_sources() -> String {
        [
            include_str!("render.rs"),
            include_str!("render_detail.rs"),
            include_str!("render_list.rs"),
            include_str!("render_summary.rs"),
        ]
        .join("\n")
    }

    #[test]
    fn datasource_browse_render_does_not_wrap_muted_shell_span() {
        let source = render_sources();
        let wrapper_signature = format!("{}{}(", "fn ", "muted");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse rendering should call tui_shell::muted directly instead of \
             carrying a local muted delegate wrapper"
        );
    }

    #[test]
    fn datasource_browse_render_does_not_wrap_boxed_shell_span() {
        let source = render_sources();
        let wrapper_signature = format!("{}{}(", "fn ", "plain_boxed");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse rendering should call tui_shell::boxed directly instead of \
             carrying a local plain_boxed delegate wrapper"
        );
    }

    #[test]
    fn datasource_browse_render_does_not_wrap_control_line_shell_rows() {
        let source = render_sources();
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
        let source = render_sources();
        let wrapper_signature = format!("{}{}(", "fn ", "build_review_lines");
        assert!(
            !source.contains(&wrapper_signature),
            "datasource browse review rendering should use a domain-specific panel builder name \
             instead of carrying a generic build_review_lines helper-drift candidate"
        );
    }
}
