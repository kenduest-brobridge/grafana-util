#![cfg(test)]
use super::*;
use crate::dashboard::delete_support::DeletePlan;

fn empty_document() -> super::super::browse_support::DashboardBrowseDocument {
    super::super::browse_support::DashboardBrowseDocument {
        summary: super::super::browse_support::DashboardBrowseSummary {
            root_path: None,
            dashboard_count: 0,
            folder_count: 0,
            org_count: 1,
            scope_label: "current-org".to_string(),
        },
        nodes: Vec::new(),
    }
}

#[test]
fn summary_lines_move_status_out_of_the_header() {
    let state = BrowserState::new(empty_document());
    let lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("Folders"));
    assert!(lines[0].contains('0'));
    assert!(lines[0].contains("Dashboards"));
    assert!(lines[1].contains("Mode"));
    assert!(lines[1].contains("browse"));
    assert!(lines[1].contains("Focus"));
    assert!(lines[1].contains("Tree"));
    assert!(!lines
        .iter()
        .any(|line| line.contains("Loaded dashboard tree")));
}

#[test]
fn summary_lines_surface_pending_delete_mode() {
    let mut state = BrowserState::new(empty_document());
    state.pending_delete = Some(DeletePlan {
        selector_uid: None,
        selector_path: None,
        delete_folders: false,
        dashboards: Vec::new(),
        folders: Vec::new(),
    });
    let lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[1].contains("Mode"));
    assert!(lines[1].contains("confirm-delete"));
    assert!(lines[1].contains("Focus"));
    assert!(lines[1].contains("Tree"));
}

#[test]
fn control_lines_use_consistent_pane_and_exit_labels() {
    let lines = control_lines(false, false, false, false)
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
fn delete_control_lines_use_cancel_labels() {
    let lines = control_lines(true, false, false, false)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("confirm delete"));
    assert!(lines[0].contains("cancel"));
    assert!(lines[1].contains("refresh"));
    assert!(!lines.iter().any(|line| line.contains("exit")));
}

#[test]
fn local_mode_summary_and_controls_mark_read_only_state() {
    let state = BrowserState::new_with_mode(empty_document(), true);
    let summary_lines = render_summary_lines(&state)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(summary_lines[1].contains("local-browse"));
    assert!(summary_lines[1].contains("Tree"));

    let lines = control_lines(false, false, false, true)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("refresh local tree"));
    assert!(lines[2].contains("read-only"));
}

#[test]
fn external_edit_control_lines_show_preview_save_apply_actions() {
    let lines = control_lines(false, false, true, false)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    assert!(lines[0].contains("apply live"));
    assert!(lines[0].contains("draft filename"));
    assert!(lines[0].contains("discard"));
    assert!(lines[1].contains("refresh preview"));
    assert!(!lines.iter().any(|line| line.contains("s ")));
}
