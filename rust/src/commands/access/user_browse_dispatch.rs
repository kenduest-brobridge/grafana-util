//! User browser general key dispatch and state transitions.

use crossterm::event::{KeyCode, KeyEvent};
use reqwest::Method;
use serde_json::Value;

use crate::access::render::map_get_text;
use crate::access::{Result, Scope, UserBrowseArgs};
use crate::common::message;

use super::user_browse_dialog::EditDialogState;
use super::user_browse_input::load_rows;
use super::user_browse_key::BrowseAction;
use super::user_browse_reload::reload_rows;
use super::user_browse_state::{row_kind, BrowserState, DisplayMode, PaneFocus, SearchDirection};

pub(super) fn handle_normal_key<F>(
    request_json: &mut F,
    args: &UserBrowseArgs,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<BrowseAction>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match key.code {
        KeyCode::BackTab => {
            state.focus_previous();
            state.status = format!(
                "Focused {} pane.",
                if state.focus == PaneFocus::List {
                    "list"
                } else {
                    "facts"
                }
            );
        }
        KeyCode::Tab => {
            state.focus_next();
            state.status = format!(
                "Focused {} pane.",
                if state.focus == PaneFocus::List {
                    "list"
                } else {
                    "facts"
                }
            );
        }
        KeyCode::Up => {
            if state.focus == PaneFocus::List {
                state.move_selection(-1);
            } else {
                let line_count = current_detail_line_count(state);
                state.move_detail_cursor(-1, line_count);
            }
        }
        KeyCode::Down => {
            if state.focus == PaneFocus::List {
                state.move_selection(1);
            } else {
                let line_count = current_detail_line_count(state);
                state.move_detail_cursor(1, line_count);
            }
        }
        KeyCode::Home => {
            if state.focus == PaneFocus::List {
                state.select_first();
            } else {
                let line_count = current_detail_line_count(state);
                state.set_detail_cursor(0, line_count);
            }
        }
        KeyCode::End => {
            if state.focus == PaneFocus::List {
                state.select_last();
            } else {
                let line_count = current_detail_line_count(state);
                state.set_detail_cursor(line_count.saturating_sub(1), line_count);
            }
        }
        KeyCode::PageUp => {
            let line_count = current_detail_line_count(state);
            state.move_detail_cursor(-10, line_count);
        }
        KeyCode::PageDown => {
            let line_count = current_detail_line_count(state);
            state.move_detail_cursor(10, line_count);
        }
        KeyCode::Right | KeyCode::Enter if state.focus == PaneFocus::List => {
            state.expand_selected();
            if state.display_mode == DisplayMode::GlobalAccounts {
                state.status = "Expanded user teams.".to_string();
            }
        }
        KeyCode::Left if state.focus == PaneFocus::List => {
            state.collapse_selected();
            if state.display_mode == DisplayMode::GlobalAccounts {
                state.status = "Collapsed user teams.".to_string();
            }
        }
        KeyCode::Char('/') => state.start_search(SearchDirection::Forward),
        KeyCode::Char('?') => state.start_search(SearchDirection::Backward),
        KeyCode::Char('n') => repeat_search(state),
        KeyCode::Char('v') => {
            if args.input_dir.is_some() {
                state.status =
                    "Local user browse keeps the account view only. Reopen a live browse for org-grouped memberships."
                        .to_string();
            } else if args.scope != Scope::Global {
                state.status =
                    "Display mode toggle is available only in global/all-org browse.".to_string();
            } else {
                state.display_mode = match state.display_mode {
                    DisplayMode::GlobalAccounts => DisplayMode::OrgMemberships,
                    DisplayMode::OrgMemberships => DisplayMode::GlobalAccounts,
                };
                state.replace_rows(load_rows(request_json, args, state.display_mode)?);
                state.status = match state.display_mode {
                    DisplayMode::GlobalAccounts => "Switched to global account view.".to_string(),
                    DisplayMode::OrgMemberships => {
                        "Switched to org-grouped membership view.".to_string()
                    }
                };
            }
        }
        KeyCode::Char('c') => {
            if state.display_mode != DisplayMode::GlobalAccounts {
                state.status =
                    "Expand/collapse all is available only in global account view.".to_string();
            } else {
                state.toggle_all_expanded();
                state.status = if state.expanded_user_ids.is_empty() {
                    "Collapsed all user team rows.".to_string()
                } else {
                    "Expanded all user team rows.".to_string()
                };
            }
        }
        KeyCode::Char('g') => {
            if args.input_dir.is_some() {
                state.status =
                    "Jumping from local user browse to team browse is unavailable. Open the team bundle directly with access team browse --input-dir ..."
                        .to_string();
            } else {
                return Ok(BrowseAction::JumpToTeam);
            }
        }
        KeyCode::Char('i') => {
            state.show_numbers = !state.show_numbers;
            state.status = if state.show_numbers {
                "Enabled row numbers in user list.".to_string()
            } else {
                "Hid row numbers in user list.".to_string()
            };
        }
        KeyCode::Char('l') => {
            reload_rows(request_json, args, state)?;
        }
        KeyCode::Char('e') => {
            if args.input_dir.is_some() {
                state.status =
                    "Local user browse is read-only. Use access user import or live browse to apply changes."
                        .to_string();
                return Ok(BrowseAction::Continue);
            }
            if state.display_mode == DisplayMode::OrgMemberships {
                state.status =
                    "Org-grouped membership view is browse-only for now. Press v for global accounts."
                        .to_string();
                return Ok(BrowseAction::Continue);
            }
            if state.selected_team_membership_row().is_some() {
                state.status = "Select a user row to edit the user.".to_string();
                return Ok(BrowseAction::Continue);
            }
            let row = state
                .selected_row()
                .ok_or_else(|| message("User browse has no selected user to edit."))?
                .clone();
            let login = map_get_text(&row, "login");
            state.pending_edit = Some(EditDialogState::new(&row));
            state.status = format!("Editing user {}.", login);
        }
        KeyCode::Char('d') => {
            if state.selected_team_membership_row().is_some() {
                if args.input_dir.is_some() {
                    state.status =
                        "Local user browse is read-only. Use access user browse against live Grafana to remove team memberships."
                            .to_string();
                } else {
                    state.pending_member_remove = true;
                    state.status = "Previewing team membership removal.".to_string();
                }
            } else if args.input_dir.is_some() {
                state.status =
                    "Local user browse is read-only. Use access user delete against live Grafana instead."
                        .to_string();
            } else if state.display_mode == DisplayMode::OrgMemberships {
                state.status =
                    "Org-grouped membership view is browse-only for now. Press v for global accounts."
                        .to_string();
            } else if state.selected_row().is_some() {
                state.pending_delete = true;
                state.status = "Previewing user delete.".to_string();
            }
        }
        KeyCode::Char('r') => {
            if state.selected_team_membership_row().is_some() {
                if args.input_dir.is_some() {
                    state.status =
                        "Local user browse is read-only. Use access user browse against live Grafana to remove team memberships."
                            .to_string();
                } else {
                    state.pending_member_remove = true;
                    state.status = "Previewing team membership removal.".to_string();
                }
            } else {
                state.status = "Select a team membership row to remove the membership.".to_string();
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => return Ok(BrowseAction::Exit),
        _ => {}
    }
    Ok(BrowseAction::Continue)
}

pub(super) fn current_detail_line_count(state: &BrowserState) -> usize {
    if state.pending_delete || state.pending_member_remove {
        return 6;
    }
    let Some(row) = state.selected_row() else {
        return 1;
    };
    match row_kind(row) {
        "org" => 4,
        "team" => 4,
        _ => 13,
    }
}

fn repeat_search(state: &mut BrowserState) {
    let Some(last) = state.last_search.clone() else {
        state.status = "No previous user search. Use / or ? first.".to_string();
        return;
    };
    if let Some(index) = state.repeat_last_search() {
        state.select_index(index);
        state.status = format!("Next match for '{}' at row {}.", last.query, index + 1);
    } else {
        state.status = format!("No more matches for '{}'.", last.query);
    }
}
