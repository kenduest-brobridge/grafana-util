//! User browser key handling and modal input routing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Method;
use serde_json::Value;

use crate::access::Result;

use super::user_browse_dialog::EditDialogAction;
use super::user_browse_mutation::{confirm_delete, confirm_member_remove, save_edit};
use super::user_browse_state::{BrowserState, SearchState};
use super::UserBrowseArgs;

pub(super) enum BrowseAction {
    Continue,
    Exit,
    JumpToTeam,
}

pub(super) fn handle_key<F>(
    request_json: &mut F,
    args: &UserBrowseArgs,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<BrowseAction>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if let Some(edit) = state.pending_edit.as_mut() {
        match edit.handle_key(key) {
            EditDialogAction::None => return Ok(BrowseAction::Continue),
            EditDialogAction::Cancel => {
                state.pending_edit = None;
                state.status = "Cancelled user edit.".to_string();
                return Ok(BrowseAction::Continue);
            }
            EditDialogAction::Save => {
                save_edit(request_json, args, state)?;
                return Ok(BrowseAction::Continue);
            }
        }
    }
    if state.pending_search.is_some() {
        handle_search_key(state, key);
        return Ok(BrowseAction::Continue);
    }
    if state.pending_delete {
        match key.code {
            KeyCode::Char('y') => {
                confirm_delete(request_json, args, state)?;
            }
            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                state.pending_delete = false;
                state.status = "Cancelled user delete.".to_string();
            }
            _ => {}
        }
        return Ok(BrowseAction::Continue);
    }
    if state.pending_member_remove {
        match key.code {
            KeyCode::Char('y') => confirm_member_remove(request_json, state)?,
            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                state.pending_member_remove = false;
                state.status = "Cancelled team membership removal.".to_string();
            }
            _ => {}
        }
        return Ok(BrowseAction::Continue);
    }

    super::user_browse_dispatch::handle_normal_key(request_json, args, state, key)
}

fn handle_search_key(state: &mut BrowserState, key: &KeyEvent) {
    let Some(mut search) = state.pending_search.take() else {
        return;
    };
    match key.code {
        KeyCode::Esc if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.status = "Cancelled user search.".to_string();
        }
        KeyCode::Enter => {
            let query = search.query.trim().to_string();
            if query.is_empty() {
                state.status = "Search query is empty.".to_string();
            } else if let Some(index) = state.find_match(&query, search.direction) {
                state.select_index(index);
                state.last_search = Some(SearchState {
                    direction: search.direction,
                    query: query.clone(),
                });
                state.status = format!("Matched '{query}' at row {}.", index + 1);
            } else {
                state.status = format!("No user matched '{query}'.");
                state.last_search = Some(SearchState {
                    direction: search.direction,
                    query,
                });
            }
        }
        KeyCode::Backspace => {
            search.query.pop();
            state.pending_search = Some(search);
        }
        KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            search.query.push(ch);
            state.pending_search = Some(search);
        }
        _ => state.pending_search = Some(search),
    }
}

#[cfg(test)]
mod tests {
    use super::super::user_browse_state::{DisplayMode, SearchDirection};
    use super::*;

    #[test]
    fn search_prompt_treats_q_as_query_text() {
        let mut state = BrowserState::new(Vec::new(), DisplayMode::GlobalAccounts);
        state.start_search(SearchDirection::Forward);

        handle_search_key(
            &mut state,
            &KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );

        assert_eq!(
            state
                .pending_search
                .as_ref()
                .map(|search| search.query.as_str()),
            Some("q")
        );
    }
}
