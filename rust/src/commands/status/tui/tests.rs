use super::*;
use crate::common::TOOL_VERSION;
use crate::project_status::{ProjectStatusFreshness, ProjectStatusOverall, PROJECT_STATUS_READY};

#[test]
fn detail_scroll_clamps_to_current_domain_lines() {
    let mut state = ProjectStatusTuiState::new(sample_project_status());
    let max_scroll = state.current_domain_lines().len().saturating_sub(1) as u16;

    state.move_detail_scroll(999);
    assert_eq!(state.detail_scroll(), max_scroll);

    state.move_detail_scroll(-999);
    assert_eq!(state.detail_scroll(), 0);
}

#[test]
fn domain_search_submit_and_repeat_wrap_selection() {
    let mut state = ProjectStatusTuiState::new(sample_project_status());

    state.start_search(SearchDirection::Forward);
    for ch in ['s', 't', 'a', 'g', 'e', 'd'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    state.handle_search_key(KeyCode::Enter);

    assert_eq!(state.pending_search(), None);
    assert_eq!(state.current_domain_index(), Some(0));
    assert_eq!(state.search_status(), "Matched 'staged' at domain 1 of 2.");

    state.repeat_search();
    assert_eq!(state.current_domain_index(), Some(1));
    assert_eq!(
        state.search_status(),
        "Next match for 'staged' at domain 2 of 2."
    );

    state.repeat_search();
    assert_eq!(state.current_domain_index(), Some(0));
    assert_eq!(
        state.search_status(),
        "Next match for 'staged' at domain 1 of 2."
    );
}

#[test]
fn action_search_uses_action_focus_and_cancel_keeps_selection() {
    let mut state = ProjectStatusTuiState::new(sample_project_status());
    state.focus = ProjectStatusPane::Actions;

    state.start_search(SearchDirection::Backward);
    assert_eq!(
        state.pending_search(),
        Some(&SearchPromptState {
            direction: SearchDirection::Backward,
            target: SearchTarget::Actions,
            query: String::new(),
        })
    );

    for ch in ['s', 'y', 'n', 'c'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    state.handle_search_key(KeyCode::Esc);

    assert_eq!(state.pending_search(), None);
    assert_eq!(state.current_action_index(), Some(0));
    assert_eq!(state.search_status(), "Cancelled status search.");
}

#[test]
fn action_search_submit_selects_matching_action_and_domain() {
    let mut state = ProjectStatusTuiState::new(sample_project_status());
    state.focus = ProjectStatusPane::Actions;

    state.start_search(SearchDirection::Forward);
    for ch in ['s', 'y', 'n', 'c'] {
        state.handle_search_key(KeyCode::Char(ch));
    }
    state.handle_search_key(KeyCode::Enter);

    assert_eq!(state.current_action_index(), Some(0));
    assert_eq!(state.current_domain_index(), Some(1));
    assert_eq!(state.search_status(), "Matched 'sync' at action 1 of 1.");
}

fn sample_project_status() -> ProjectStatus {
    ProjectStatus {
        schema_version: 1,
        tool_version: TOOL_VERSION.to_string(),
        discovery: None,
        scope: "live".to_string(),
        overall: ProjectStatusOverall {
            status: PROJECT_STATUS_READY.to_string(),
            domain_count: 2,
            present_count: 2,
            blocked_count: 1,
            blocker_count: 3,
            warning_count: 0,
            freshness: ProjectStatusFreshness {
                status: "current".to_string(),
                source_count: 1,
                newest_age_seconds: Some(30),
                oldest_age_seconds: Some(30),
            },
        },
        domains: vec![
            ProjectDomainStatus {
                id: "dashboard".to_string(),
                scope: "staged".to_string(),
                mode: "inspect-summary".to_string(),
                status: PROJECT_STATUS_READY.to_string(),
                reason_code: PROJECT_STATUS_READY.to_string(),
                primary_count: 4,
                blocker_count: 0,
                warning_count: 0,
                source_kinds: vec!["dashboard-export".to_string()],
                signal_keys: vec!["summary.dashboardCount".to_string()],
                blockers: Vec::new(),
                warnings: Vec::new(),
                next_actions: vec!["review dashboard governance warnings".to_string()],
                freshness: ProjectStatusFreshness {
                    status: "current".to_string(),
                    source_count: 1,
                    newest_age_seconds: Some(30),
                    oldest_age_seconds: Some(30),
                },
            },
            ProjectDomainStatus {
                id: "sync".to_string(),
                scope: "staged".to_string(),
                mode: "staged-documents".to_string(),
                status: PROJECT_STATUS_BLOCKED.to_string(),
                reason_code: "blocked-by-blockers".to_string(),
                primary_count: 6,
                blocker_count: 3,
                warning_count: 0,
                source_kinds: vec!["sync-summary".to_string()],
                signal_keys: vec!["summary.syncBlockingCount".to_string()],
                blockers: vec![crate::project_status::status_finding(
                    "sync-blocking",
                    3,
                    "summary.syncBlockingCount",
                )],
                warnings: Vec::new(),
                next_actions: vec!["resolve sync workflow blockers".to_string()],
                freshness: ProjectStatusFreshness {
                    status: "current".to_string(),
                    source_count: 1,
                    newest_age_seconds: Some(10),
                    oldest_age_seconds: Some(10),
                },
            },
        ],
        top_blockers: Vec::new(),
        next_actions: vec![ProjectStatusAction {
            domain: "sync".to_string(),
            reason_code: "blocked-by-blockers".to_string(),
            action: "resolve sync workflow blockers".to_string(),
        }],
    }
}
