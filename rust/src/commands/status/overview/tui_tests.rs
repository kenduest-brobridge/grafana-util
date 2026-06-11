use super::*;
use crate::overview::OverviewSectionFact;
use crate::overview::OverviewSummary;
use crate::project_status::{ProjectDomainStatus, ProjectStatus, ProjectStatusOverall};
use ratatui::Terminal;

fn test_section(kind: &str, label: &str, subtitle: &str) -> OverviewSection {
    OverviewSection {
        artifact_index: 0,
        kind: kind.to_string(),
        label: label.to_string(),
        subtitle: subtitle.to_string(),
        views: vec![OverviewSectionView {
            label: "Summary".to_string(),
            items: vec![OverviewSectionItem {
                kind: "test".to_string(),
                title: label.to_string(),
                meta: "meta".to_string(),
                facts: vec![],
                details: vec![],
            }],
        }],
    }
}

fn search_document() -> OverviewDocument {
    OverviewDocument {
        kind: "grafana-utils-overview".to_string(),
        schema_version: 1,
        tool_version: crate::common::TOOL_VERSION.to_string(),
        discovery: None,
        summary: OverviewSummary::default(),
        project_status: ProjectStatus {
            schema_version: 1,
            tool_version: crate::common::TOOL_VERSION.to_string(),
            discovery: None,
            scope: "staged-only".to_string(),
            overall: ProjectStatusOverall {
                status: "ready".to_string(),
                domain_count: 1,
                present_count: 1,
                blocked_count: 0,
                blocker_count: 0,
                warning_count: 0,
                freshness: Default::default(),
            },
            domains: vec![ProjectDomainStatus {
                id: "dashboard".to_string(),
                scope: "staged".to_string(),
                mode: "artifact-summary".to_string(),
                status: "ready".to_string(),
                reason_code: "ready".to_string(),
                primary_count: 1,
                blocker_count: 0,
                warning_count: 0,
                source_kinds: vec!["dashboard-export".to_string()],
                signal_keys: vec![],
                blockers: vec![],
                warnings: vec![],
                next_actions: vec![],
                freshness: Default::default(),
            }],
            top_blockers: vec![],
            next_actions: vec![],
        },
        artifacts: vec![],
        selected_section_index: 0,
        sections: vec![OverviewSection {
            artifact_index: 0,
            kind: "dashboard-export".to_string(),
            label: "Dashboard export".to_string(),
            subtitle: "dashboards=3".to_string(),
            views: vec![OverviewSectionView {
                label: "Current".to_string(),
                items: vec![
                    OverviewSectionItem {
                        kind: "dashboard".to_string(),
                        title: "Alpha blocker".to_string(),
                        meta: "status=blocked".to_string(),
                        facts: vec![OverviewSectionFact {
                            label: "uid".to_string(),
                            value: "alpha".to_string(),
                        }],
                        details: vec!["owner=platform".to_string()],
                    },
                    OverviewSectionItem {
                        kind: "dashboard".to_string(),
                        title: "Beta ready".to_string(),
                        meta: "status=ready".to_string(),
                        facts: vec![OverviewSectionFact {
                            label: "uid".to_string(),
                            value: "beta".to_string(),
                        }],
                        details: vec!["owner=ops".to_string()],
                    },
                    OverviewSectionItem {
                        kind: "warning".to_string(),
                        title: "Gamma blocker".to_string(),
                        meta: "status=blocked".to_string(),
                        facts: vec![OverviewSectionFact {
                            label: "uid".to_string(),
                            value: "gamma".to_string(),
                        }],
                        details: vec!["owner=platform".to_string()],
                    },
                ],
            }],
        }],
    }
}

pub(crate) fn test_document() -> OverviewDocument {
    OverviewDocument {
        kind: "grafana-utils-overview".to_string(),
        schema_version: 1,
        tool_version: crate::common::TOOL_VERSION.to_string(),
        discovery: None,
        summary: OverviewSummary::default(),
        project_status: ProjectStatus {
            schema_version: 1,
            tool_version: crate::common::TOOL_VERSION.to_string(),
            discovery: None,
            scope: "staged-only".to_string(),
            overall: ProjectStatusOverall {
                status: "blocked".to_string(),
                domain_count: 6,
                present_count: 2,
                blocked_count: 1,
                blocker_count: 2,
                warning_count: 0,
                freshness: Default::default(),
            },
            domains: vec![
                ProjectDomainStatus {
                    id: "dashboard".to_string(),
                    scope: "staged".to_string(),
                    mode: "artifact-summary".to_string(),
                    status: "ready".to_string(),
                    reason_code: "ready".to_string(),
                    primary_count: 1,
                    blocker_count: 0,
                    warning_count: 0,
                    source_kinds: vec!["dashboard-export".to_string()],
                    signal_keys: vec!["summary.dashboardCount".to_string()],
                    blockers: vec![],
                    warnings: vec![],
                    next_actions: vec![],
                    freshness: Default::default(),
                },
                ProjectDomainStatus {
                    id: "sync".to_string(),
                    scope: "staged".to_string(),
                    mode: "artifact-summary".to_string(),
                    status: "blocked".to_string(),
                    reason_code: "blocked-by-blockers".to_string(),
                    primary_count: 1,
                    blocker_count: 2,
                    warning_count: 0,
                    source_kinds: vec!["sync-summary".to_string()],
                    signal_keys: vec!["summary.blockingCount".to_string()],
                    blockers: vec![],
                    warnings: vec![],
                    next_actions: vec![
                        "resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact".to_string(),
                    ],
                    freshness: Default::default(),
                },
            ],
            top_blockers: vec![],
            next_actions: vec![],
        },
        artifacts: vec![],
        selected_section_index: 0,
        sections: vec![
            test_section("dashboard-export", "Dashboard export", "dashboards=1"),
            test_section("bundle-preflight", "Sync bundle preflight", "blocking=2"),
        ],
    }
}

#[test]
fn overview_tui_starts_on_items_so_arrow_keys_move_rows_immediately() {
    let mut state = OverviewWorkbenchState::new(search_document());

    assert_eq!(state.focus, OverviewPane::Items);
    assert_eq!(state.item_state.selected(), Some(0));

    if state.focus == OverviewPane::Items {
        state.move_item_selection(1);
    }

    assert_eq!(state.item_state.selected(), Some(1));
}

#[test]
fn project_home_is_available_and_hands_off_to_first_blocked_section() {
    let mut state = OverviewWorkbenchState::new(test_document());

    assert_eq!(state.focus, OverviewPane::Items);
    assert_eq!(
        state.project_home_target_label().as_deref(),
        Some("Sync bundle preflight")
    );

    state.focus_project_home();
    assert_eq!(state.focus, OverviewPane::ProjectHome);
    state.focus_next();
    assert_eq!(state.focus, OverviewPane::Sections);
    state.focus_previous();
    assert_eq!(state.focus, OverviewPane::ProjectHome);

    state.handoff_from_home();
    assert_eq!(state.focus, OverviewPane::Sections);
    assert_eq!(state.section_state.selected(), Some(1));
    assert_eq!(
        state
            .current_section()
            .map(|section| section.label.as_str()),
        Some("Sync bundle preflight")
    );
}

#[test]
fn project_home_lines_surface_status_and_next_action() {
    let state = OverviewWorkbenchState::new(test_document());
    let lines = state.project_home_lines().join("\n");

    assert!(lines.contains("Overall: status=blocked"));
    assert!(lines.contains("Recommended handoff section: Sync bundle preflight"));
    assert!(lines.contains("Top action: sync status=blocked reason=blocked-by-blockers"));
    assert!(lines.contains("next=resolve sync workflow blockers in the fixed order: sync, provider, secret-placeholder, alert-artifact"));
    assert!(!lines.contains("Navigation: Enter hands off from Home"));
    assert!(lines.contains("Domains: dashboard=ready | sync=blocked"));
}

#[test]
fn interactive_render_starts_on_project_home_surface() {
    use ratatui::backend::TestBackend;

    let mut state = OverviewWorkbenchState::new(test_document());
    let mut terminal = Terminal::new(TestBackend::new(180, 40)).unwrap();

    terminal
        .draw(|frame| overview_tui_render::render_overview_frame(frame, &mut state))
        .unwrap();

    let screen = format!("{}", terminal.backend());
    assert!(screen.contains("Overview"));
    assert!(screen.contains("Recommended handoff section: Sync bundle preflight"));
    assert!(screen.contains("Status & Controls"));
    assert!(!screen.contains("Project Home [Focused]"));
    assert_eq!(state.focus, OverviewPane::Items);
}

#[test]
fn search_prompt_submit_and_cancel_update_local_search_state() {
    let mut state = OverviewWorkbenchState::new(search_document());

    state.start_search(SearchDirection::Forward);
    assert_eq!(
        state.pending_search,
        Some(SearchPromptState {
            direction: SearchDirection::Forward,
            query: String::new(),
        })
    );
    assert_eq!(
        state.search_status,
        "Search forward within the current view items."
    );

    for ch in ['b', 'l', 'o', 'c', 'k'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    assert_eq!(
        state
            .pending_search
            .as_ref()
            .map(|search| search.query.as_str()),
        Some("block")
    );

    state.handle_search_key(KeyCode::Enter);
    assert_eq!(state.pending_search, None);
    assert_eq!(state.item_state.selected(), Some(0));
    assert_eq!(
        state.last_search,
        Some(SearchState {
            direction: SearchDirection::Forward,
            query: "block".to_string(),
        })
    );
    assert_eq!(
        state.search_status,
        "Matched 'block' at item 1 of 3 in the current view."
    );

    state.start_search(SearchDirection::Backward);
    state.handle_search_key(KeyCode::Esc);
    assert_eq!(state.pending_search, None);
    assert_eq!(state.search_status, "Cancelled status overview search.");
}

#[test]
fn search_prompt_summary_tracks_pending_and_last_search_state() {
    let mut state = OverviewWorkbenchState::new(search_document());

    assert_eq!(state.search_summary(), "idle");

    state.item_state.select(Some(2));
    state.start_search(SearchDirection::Backward);
    assert_eq!(state.search_summary(), "prompt ?");

    state.handle_search_key(KeyCode::Char('g'));
    state.handle_search_key(KeyCode::Char('a'));
    state.handle_search_key(KeyCode::Char('m'));
    state.handle_search_key(KeyCode::Char('m'));
    state.handle_search_key(KeyCode::Char('x'));
    state.handle_search_key(KeyCode::Backspace);
    assert_eq!(state.search_summary(), "prompt ?gamm");

    state.handle_search_key(KeyCode::Enter);
    assert_eq!(state.search_summary(), "last ?gamm");
    assert_eq!(state.item_state.selected(), Some(2));
}

#[test]
fn repeat_search_wraps_within_current_view_items() {
    let mut state = OverviewWorkbenchState::new(search_document());

    state.start_search(SearchDirection::Forward);
    for ch in ['b', 'l', 'o', 'c', 'k'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    state.handle_search_key(KeyCode::Enter);
    assert_eq!(state.item_state.selected(), Some(0));

    state.repeat_search();
    assert_eq!(state.item_state.selected(), Some(2));
    assert_eq!(
        state.search_status,
        "Next match for 'block' at item 3 of 3 in the current view."
    );

    state.repeat_search();
    assert_eq!(state.item_state.selected(), Some(0));
    assert_eq!(
        state.search_status,
        "Next match for 'block' at item 1 of 3 in the current view."
    );
}

#[test]
fn backward_repeat_search_wraps_within_current_view_items() {
    let mut state = OverviewWorkbenchState::new(search_document());

    state.item_state.select(Some(2));
    state.start_search(SearchDirection::Backward);
    for ch in ['b', 'l', 'o', 'c', 'k'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    state.handle_search_key(KeyCode::Enter);
    assert_eq!(state.item_state.selected(), Some(2));

    state.repeat_search();
    assert_eq!(state.item_state.selected(), Some(0));
    assert_eq!(
        state.search_status,
        "Next match for 'block' at item 1 of 3 in the current view."
    );

    state.repeat_search();
    assert_eq!(state.item_state.selected(), Some(2));
    assert_eq!(
        state.search_status,
        "Next match for 'block' at item 3 of 3 in the current view."
    );
}
