#![cfg(feature = "tui")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Method;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{message, Result};

use super::browse_actions::{
    apply_dashboard_edit_save, apply_external_dashboard_edit, begin_dashboard_edit,
    begin_dashboard_history, begin_external_dashboard_edit, build_delete_preview,
    delete_status_message, execute_delete_plan_with_request, load_live_detail_lines,
    refresh_browser_document, restore_dashboard_history_version,
};
use super::browse_edit_dialog::EditDialogAction;
use super::browse_external_edit_dialog::ExternalEditDialogAction;
use super::browse_history_dialog::HistoryDialogAction;
use super::browse_render::render_dashboard_browser_frame;
use super::browse_state::{
    BrowserState, CompletionNotice, PaneFocus, SearchDirection, SearchState,
};
use super::browse_support::{DashboardBrowseNode, DashboardBrowseNodeKind};
use super::browse_terminal::TerminalSession;
use super::import::collect_import_dry_run_report_with_request;
use super::{
    build_http_client_for_org, BrowseArgs, CommonCliArgs, DashboardImportInputFormat, ImportArgs,
};

pub(crate) enum BrowserLoopAction {
    Continue,
    Exit,
}

pub(crate) fn handle_browser_key<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    session: &mut TerminalSession,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<BrowserLoopAction>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if state.completion_notice.is_some() {
        state.completion_notice = None;
        return Ok(BrowserLoopAction::Continue);
    }
    if state.pending_history.is_some() {
        handle_history_dialog_key(request_json, args, session, state, key)?;
        return Ok(BrowserLoopAction::Continue);
    }
    if state.pending_search.is_some() {
        handle_search_dialog_key(state, key)?;
        return Ok(BrowserLoopAction::Continue);
    }
    if state.pending_external_edit.is_some() {
        handle_external_edit_dialog_key(request_json, args, session, state, key)?;
        return Ok(BrowserLoopAction::Continue);
    }
    if state.pending_edit.is_some() {
        handle_edit_dialog_key(request_json, args, state, key)?;
        return Ok(BrowserLoopAction::Continue);
    }

    match key.code {
        KeyCode::BackTab if state.pending_delete.is_none() => {
            state.focus_previous_pane();
            state.status = format!("Focused {} pane.", state.focus_label());
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Tab if state.pending_delete.is_none() => {
            state.focus_next_pane();
            state.status = format!("Focused {} pane.", state.focus_label());
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Esc => {
            if state.pending_delete.is_some() {
                state.pending_delete = None;
                state.detail_scroll = 0;
                state.status = "Cancelled delete preview.".to_string();
                Ok(BrowserLoopAction::Continue)
            } else {
                Ok(BrowserLoopAction::Exit)
            }
        }
        KeyCode::Char('q') => Ok(BrowserLoopAction::Exit),
        KeyCode::Up if state.pending_delete.is_none() => {
            if state.focus == PaneFocus::Tree {
                state.move_selection(-1);
                state.detail_scroll = 0;
                ensure_selected_dashboard_view(request_json, args, state, false)?;
            } else {
                state.detail_scroll = state.detail_scroll.saturating_sub(1);
            }
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Down if state.pending_delete.is_none() => {
            if state.focus == PaneFocus::Tree {
                state.move_selection(1);
                state.detail_scroll = 0;
                ensure_selected_dashboard_view(request_json, args, state, false)?;
            } else {
                state.detail_scroll = state.detail_scroll.saturating_add(1);
            }
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Home if state.pending_delete.is_none() => {
            if state.focus == PaneFocus::Tree {
                state.select_first();
                state.detail_scroll = 0;
                ensure_selected_dashboard_view(request_json, args, state, false)?;
            } else {
                state.detail_scroll = 0;
            }
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::End if state.pending_delete.is_none() => {
            if state.focus == PaneFocus::Tree {
                state.select_last();
                state.detail_scroll = 0;
                ensure_selected_dashboard_view(request_json, args, state, false)?;
            } else {
                state.detail_scroll = u16::MAX.saturating_sub(32);
            }
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::PageUp => {
            state.detail_scroll = state.detail_scroll.saturating_sub(8);
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::PageDown => {
            state.detail_scroll = state.detail_scroll.saturating_add(8);
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('l') => {
            let document = refresh_browser_document(request_json, args)?;
            state.replace_document(document);
            state.status = "Refreshed dashboard tree.".to_string();
            ensure_selected_dashboard_view(request_json, args, state, false)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('/') if state.pending_delete.is_none() => {
            state.start_search(SearchDirection::Forward);
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('?') if state.pending_delete.is_none() => {
            state.start_search(SearchDirection::Backward);
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('n') if state.pending_delete.is_none() => {
            repeat_search(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('v') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status = "Local browse shows tree facts from export files. Live dashboard details are unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            refresh_selected_dashboard_view(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('h') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status = "Local browse does not support live history browsing.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            open_selected_dashboard_history(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('r')
            if state.pending_delete.is_none() && !key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            if state.local_mode {
                state.status = "Local browse is read-only. Rename is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            start_selected_dashboard_rename(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('m') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status = "Local browse is read-only. Move is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            start_selected_dashboard_move(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('e')
            if state.pending_delete.is_none() && !key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            if state.local_mode {
                state.status = "Local browse is read-only. Edit dialog is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            start_selected_dashboard_edit(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('E') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status =
                    "Local browse is read-only. Raw JSON edit is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            run_selected_external_edit(request_json, args, session, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('e')
            if state.pending_delete.is_none() && key.modifiers.contains(KeyModifiers::SHIFT) =>
        {
            if state.local_mode {
                state.status =
                    "Local browse is read-only. Raw JSON edit is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            run_selected_external_edit(request_json, args, session, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('d') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status = "Local browse is read-only. Delete is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            preview_selected_delete(request_json, args, state, false)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('D') if state.pending_delete.is_none() => {
            if state.local_mode {
                state.status = "Local browse is read-only. Delete is unavailable.".to_string();
                return Ok(BrowserLoopAction::Continue);
            }
            let include_folders = matches!(
                state.selected_node().map(|node| node.kind.clone()),
                Some(DashboardBrowseNodeKind::Folder)
            );
            preview_selected_delete(request_json, args, state, include_folders)?;
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('n') if state.pending_delete.is_some() => {
            state.pending_delete = None;
            state.detail_scroll = 0;
            state.status = "Cancelled delete preview.".to_string();
            Ok(BrowserLoopAction::Continue)
        }
        KeyCode::Char('y') if state.pending_delete.is_some() => {
            confirm_delete(request_json, args, state)?;
            Ok(BrowserLoopAction::Continue)
        }
        _ => Ok(BrowserLoopAction::Continue),
    }
}

fn handle_search_dialog_key(state: &mut BrowserState, key: &KeyEvent) -> Result<()> {
    let mut search = state
        .pending_search
        .take()
        .ok_or_else(|| message("Dashboard browse search state is missing."))?;
    match key.code {
        KeyCode::Esc if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.status = "Cancelled dashboard search.".to_string();
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
                state.status = format!("Matched '{query}' at tree row {}.", index + 1);
            } else {
                state.last_search = Some(SearchState {
                    direction: search.direction,
                    query: query.clone(),
                });
                state.status = format!("No org, folder, or dashboard matched '{query}'.");
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
        _ => {
            state.pending_search = Some(search);
        }
    }
    Ok(())
}

fn handle_history_dialog_key<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    session: &mut TerminalSession,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(history_state) = state.pending_history.as_mut() else {
        return Ok(());
    };
    match history_state.handle_key(key) {
        HistoryDialogAction::Continue => {}
        HistoryDialogAction::Close => {
            state.pending_history = None;
            state.status = "Closed dashboard history.".to_string();
        }
        HistoryDialogAction::Restore {
            uid,
            version,
            message,
        } => {
            let Some(node) = state.selected_node().cloned() else {
                return Ok(());
            };
            if let Some(dialog) = state.pending_history.as_mut() {
                dialog.set_busy_message(format!(
                    "Working... restoring {} to version {}.",
                    uid, version
                ));
            }
            state.status = format!("Restoring {} to version {}...", uid, version);
            redraw_browser(session, state)?;
            if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                restore_dashboard_history_version(&mut scoped, &uid, version, &message)?;
            } else {
                restore_dashboard_history_version(request_json, &uid, version, &message)?;
            }
            state.pending_history = None;
            let document = refresh_browser_document(request_json, args)?;
            state.replace_document(document);
            state.status = format!("Restored dashboard {} to version {}.", uid, version);
            ensure_selected_dashboard_view(request_json, args, state, false)?;
        }
    }
    Ok(())
}

fn handle_edit_dialog_key<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(edit_state) = state.pending_edit.as_mut() else {
        return Ok(());
    };
    match edit_state.handle_key(key) {
        EditDialogAction::Continue => {}
        EditDialogAction::Cancelled => {
            state.pending_edit = None;
            state.status = "Cancelled dashboard edit.".to_string();
        }
        EditDialogAction::Save { draft, update } => {
            let Some(node) = state.selected_node().cloned() else {
                return Ok(());
            };
            state.pending_edit = None;
            let applied = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                apply_dashboard_edit_save(&mut scoped, &state.document, &draft, &update)?
            } else {
                apply_dashboard_edit_save(request_json, &state.document, &draft, &update)?
            };
            if !applied {
                state.status = format!("No dashboard changes to apply for {}.", draft.uid);
                return Ok(());
            }
            let document = refresh_browser_document(request_json, args)?;
            state.replace_document(document);
            state.status = format!("Updated dashboard {}.", draft.uid);
            ensure_selected_dashboard_view(request_json, args, state, false)?;
        }
    }
    Ok(())
}

fn handle_external_edit_dialog_key<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    session: &mut TerminalSession,
    state: &mut BrowserState,
    key: &KeyEvent,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(action) = state
        .pending_external_edit
        .as_mut()
        .map(|dialog| dialog.handle_key(key))
    else {
        return Ok(());
    };
    match action {
        ExternalEditDialogAction::Continue => {}
        ExternalEditDialogAction::Close => {
            let uid = state
                .pending_external_edit
                .as_ref()
                .map(|dialog| dialog.uid.clone())
                .unwrap_or_else(|| "dashboard".to_string());
            state.pending_external_edit = None;
            state.status = format!("Discarded raw JSON edit review for {}.", uid);
        }
        ExternalEditDialogAction::SaveDraft(save_path) => {
            let Some(dialog) = state.pending_external_edit.as_ref() else {
                return Ok(());
            };
            let uid = dialog.uid.clone();
            let updated_payload = dialog.updated_payload.clone();
            if let Some(dialog) = state.pending_external_edit.as_mut() {
                dialog.set_busy_message(format!(
                    "Working... writing draft file to {}.",
                    save_path.display()
                ));
            }
            state.status = format!("Writing draft file for {}...", uid);
            redraw_browser(session, state)?;
            if let Some(parent) = save_path
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                &save_path,
                serde_json::to_string_pretty(&updated_payload)? + "\n",
            )?;
            state.pending_external_edit = None;
            state.status = format!(
                "Wrote raw JSON draft for {} to {}.",
                uid,
                save_path.display()
            );
        }
        ExternalEditDialogAction::PreviewDryRun => {
            let payload = state
                .pending_external_edit
                .as_ref()
                .map(|dialog| dialog.updated_payload.clone())
                .ok_or_else(|| message("Raw JSON edit review state disappeared."))?;
            let uid = state
                .pending_external_edit
                .as_ref()
                .map(|dialog| dialog.uid.clone())
                .unwrap_or_else(|| "dashboard".to_string());
            if let Some(dialog) = state.pending_external_edit.as_mut() {
                dialog.set_busy_message(format!("Working... refreshing live preview for {}.", uid));
            }
            state.status = format!("Refreshing live preview for {}...", uid);
            redraw_browser(session, state)?;
            let Some(node) = state.selected_node().cloned() else {
                return Ok(());
            };
            let preview_lines = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                preview_external_edit_dry_run(&mut scoped, args, &payload)?
            } else {
                preview_external_edit_dry_run(request_json, args, &payload)?
            };
            if let Some(dialog) = state.pending_external_edit.as_mut() {
                dialog.clear_busy_message();
                dialog.preview_lines = Some(preview_lines);
            }
            state.status = format!("Refreshed live preview for {}.", uid);
        }
        ExternalEditDialogAction::ApplyLive => {
            let payload = state
                .pending_external_edit
                .as_ref()
                .map(|dialog| dialog.updated_payload.clone())
                .ok_or_else(|| message("Raw JSON edit review state disappeared."))?;
            let uid = state
                .pending_external_edit
                .as_ref()
                .map(|dialog| dialog.uid.clone())
                .unwrap_or_else(|| "dashboard".to_string());
            if let Some(dialog) = state.pending_external_edit.as_mut() {
                dialog.set_busy_message(format!("Working... applying live edit for {}.", uid));
            }
            state.status = format!("Applying live edit for {}...", uid);
            redraw_browser(session, state)?;
            let Some(node) = state.selected_node().cloned() else {
                return Ok(());
            };
            state.pending_external_edit = None;
            if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                apply_external_dashboard_edit(&mut scoped, &payload)?;
            } else {
                apply_external_dashboard_edit(request_json, &payload)?;
            }
            let document = refresh_browser_document(request_json, args)?;
            state.replace_document(document);
            state.status = format!("Applied live edit for dashboard {}.", uid);
            state.completion_notice = Some(CompletionNotice {
                title: "Applied".to_string(),
                body: format!("Updated live dashboard {} successfully.", uid),
            });
            ensure_selected_dashboard_view(request_json, args, state, false)?;
        }
    }
    Ok(())
}

fn redraw_browser(session: &mut TerminalSession, state: &mut BrowserState) -> Result<()> {
    session
        .terminal
        .draw(|frame| render_dashboard_browser_frame(frame, state))?;
    Ok(())
}

pub(crate) fn ensure_selected_dashboard_view<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
    announce: bool,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org => {
            if announce {
                state.status = if state.local_mode {
                    "Org rows summarize the local export scope. Select a folder or dashboard row."
                        .to_string()
                } else {
                    "Org rows summarize browse scope. Select a folder or dashboard row.".to_string()
                };
            }
        }
        DashboardBrowseNodeKind::Dashboard => {
            if state.local_mode {
                if announce {
                    state.status =
                        "Local browse shows dashboard facts from export files. Live details and actions are unavailable."
                            .to_string();
                }
                state.detail_scroll = 0;
                return Ok(());
            }
            if let Some(cache_key) = live_view_cache_key(&node) {
                if state.live_view_cache.contains_key(&cache_key) {
                    return Ok(());
                }
            }
            let lines = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                load_live_detail_lines(&mut scoped, &node)?
            } else {
                load_live_detail_lines(request_json, &node)?
            };
            if let Some(cache_key) = live_view_cache_key(&node) {
                state.live_view_cache.insert(cache_key, lines);
            }
            state.detail_scroll = 0;
            if announce {
                state.status = format!("Loaded live dashboard details for {}.", node.title);
            }
        }
        DashboardBrowseNodeKind::Folder => {
            if announce {
                state.status = if state.local_mode {
                    "Folder rows already show local tree metadata.".to_string()
                } else {
                    "Folder rows already show live tree metadata.".to_string()
                };
            }
        }
    }
    Ok(())
}

fn refresh_selected_dashboard_view<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    if let Some(node) = state.selected_node().cloned() {
        if let Some(cache_key) = live_view_cache_key(&node) {
            state.live_view_cache.remove(&cache_key);
        }
    }
    ensure_selected_dashboard_view(request_json, args, state, true)
}

fn open_selected_dashboard_history<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org | DashboardBrowseNodeKind::Folder => {
            state.status = "History is only available for dashboard rows.".to_string();
        }
        DashboardBrowseNodeKind::Dashboard => {
            state.pending_history = Some(if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                begin_dashboard_history(&mut scoped, &node)?
            } else {
                begin_dashboard_history(request_json, &node)?
            });
            state.status = format!("Viewing dashboard history for {}.", node.title);
        }
    }
    Ok(())
}

fn start_selected_dashboard_edit<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org | DashboardBrowseNodeKind::Folder => {
            state.status =
                "Folder/org edit is not available in v2 yet. Select a dashboard row.".to_string();
        }
        DashboardBrowseNodeKind::Dashboard => {
            state.pending_edit = Some(if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                begin_dashboard_edit(&mut scoped, &state.document, &node)?
            } else {
                begin_dashboard_edit(request_json, &state.document, &node)?
            });
            state.status =
                "Editing dashboard in TUI dialog. Ctrl+S saves, Esc cancels.".to_string();
        }
    }
    Ok(())
}

fn start_selected_dashboard_rename<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org | DashboardBrowseNodeKind::Folder => {
            state.status = "Rename is only available for dashboard rows right now.".to_string();
        }
        DashboardBrowseNodeKind::Dashboard => {
            let mut dialog = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                begin_dashboard_edit(&mut scoped, &state.document, &node)?
            } else {
                begin_dashboard_edit(request_json, &state.document, &node)?
            };
            dialog.focus_title_rename();
            state.pending_edit = Some(dialog);
            state.status = "Rename dashboard in TUI dialog. Ctrl+S saves, Esc cancels.".to_string();
        }
    }
    Ok(())
}

fn start_selected_dashboard_move<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org | DashboardBrowseNodeKind::Folder => {
            state.status = "Move is only available for dashboard rows right now.".to_string();
        }
        DashboardBrowseNodeKind::Dashboard => {
            let mut dialog = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                begin_dashboard_edit(&mut scoped, &state.document, &node)?
            } else {
                begin_dashboard_edit(request_json, &state.document, &node)?
            };
            dialog.focus_folder_move();
            state.pending_edit = Some(dialog);
            state.status =
                "Move dashboard to another folder. Choose a folder, then Ctrl+S saves.".to_string();
        }
    }
    Ok(())
}

fn run_selected_external_edit<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    session: &mut TerminalSession,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    match node.kind {
        DashboardBrowseNodeKind::Org | DashboardBrowseNodeKind::Folder => {
            state.status = "Raw JSON edit is only available for dashboard rows.".to_string();
        }
        DashboardBrowseNodeKind::Dashboard => {
            session.suspend()?;
            let raw_result = if let Some(client) = scoped_org_client(args, &node)? {
                let mut scoped = |method: Method,
                                  path: &str,
                                  params: &[(String, String)],
                                  payload: Option<&Value>|
                 -> Result<Option<Value>> {
                    client.request_json(method, path, params, payload)
                };
                begin_external_dashboard_edit(&mut scoped, &node)
            } else {
                begin_external_dashboard_edit(request_json, &node)
            };
            session.resume()?;
            if let Some(dialog) = raw_result? {
                let uid = dialog.uid.clone();
                state.status = format!("Preparing live preview for raw JSON edit on {}...", uid);
                let preview_lines = if let Some(client) = scoped_org_client(args, &node)? {
                    let mut scoped = |method: Method,
                                      path: &str,
                                      params: &[(String, String)],
                                      payload: Option<&Value>|
                     -> Result<Option<Value>> {
                        client.request_json(method, path, params, payload)
                    };
                    preview_external_edit_dry_run(&mut scoped, args, &dialog.updated_payload)?
                } else {
                    preview_external_edit_dry_run(request_json, args, &dialog.updated_payload)?
                };
                let mut dialog = dialog;
                dialog.preview_lines = Some(preview_lines);
                state.pending_external_edit = Some(dialog);
                state.status = format!(
                    "Review raw JSON edit for {}. Preview is ready. a applies live, w opens draft filename input, q discards.",
                    uid
                );
            } else {
                state.status = format!("Raw JSON edit cancelled or unchanged for {}.", node.title);
            }
        }
    }
    Ok(())
}

fn preview_external_edit_temp_dir(uid: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "grafana-dashboard-browse-preview-{}-{}-{}",
        process::id(),
        uid,
        timestamp
    ))
}

fn build_external_edit_import_args(args: &BrowseArgs, input_dir: PathBuf) -> ImportArgs {
    ImportArgs {
        common: CommonCliArgs {
            color: args.common.color,
            profile: args.common.profile.clone(),
            url: args.common.url.clone(),
            api_token: args.common.api_token.clone(),
            username: args.common.username.clone(),
            password: args.common.password.clone(),
            prompt_password: args.common.prompt_password,
            prompt_token: args.common.prompt_token,
            timeout: args.common.timeout,
            verify_ssl: args.common.verify_ssl,
        },
        org_id: None,
        use_export_org: false,
        only_org_id: Vec::new(),
        create_missing_orgs: false,
        input_dir,
        input_format: DashboardImportInputFormat::Raw,
        import_folder_uid: None,
        ensure_folders: false,
        replace_existing: true,
        update_existing_only: false,
        require_matching_folder_path: false,
        require_matching_export_org: false,
        strict_schema: false,
        target_schema_version: None,
        import_message: "Preview raw JSON edit from dashboard browse".to_string(),
        interactive: false,
        dry_run: true,
        table: true,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    }
}

fn preview_external_edit_dry_run<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    updated_payload: &Value,
) -> Result<Vec<String>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let uid = updated_payload
        .get("dashboard")
        .and_then(Value::as_object)
        .and_then(|dashboard| dashboard.get("uid"))
        .and_then(Value::as_str)
        .unwrap_or("dashboard");
    let temp_dir = preview_external_edit_temp_dir(uid);
    fs::create_dir_all(&temp_dir)?;
    let result = (|| -> Result<Vec<String>> {
        let staged_path = temp_dir.join("dashboard.json");
        fs::write(
            &staged_path,
            serde_json::to_string_pretty(updated_payload)? + "\n",
        )?;
        let import_args = build_external_edit_import_args(args, temp_dir.clone());
        let report = collect_import_dry_run_report_with_request(&mut *request_json, &import_args)?;
        Ok(render_external_edit_preview_lines(
            &report.mode,
            &report.dashboard_records,
        ))
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn render_external_edit_preview_lines(
    mode: &str,
    dashboard_records: &[[String; 8]],
) -> Vec<String> {
    let mut lines = vec![
        "Preview only. Nothing has been written to Grafana yet.".to_string(),
        format!("Mode: {mode}"),
    ];
    if dashboard_records.is_empty() {
        lines.push("No dashboard changes were staged for import preview.".to_string());
        return lines;
    }
    for row in dashboard_records {
        let uid = row[0].as_str();
        let destination = row[1].as_str();
        let action = row[2].as_str();
        let folder_path = if row[3].trim().is_empty() {
            "-"
        } else {
            row[3].as_str()
        };
        lines.push(String::new());
        lines.push(format!("Dashboard: {uid}"));
        lines.push(format!("Destination: {destination}"));
        lines.push(format!("Action: {action}"));
        lines.push(format!("Folder: {folder_path}"));
        if !row[6].trim().is_empty() {
            lines.push(format!("Reason: {}", row[6]));
        }
    }
    lines
}

fn preview_selected_delete<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
    include_folders: bool,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    if node.kind == DashboardBrowseNodeKind::Org {
        state.status =
            "Org rows do not support delete. Select a folder or dashboard row.".to_string();
        return Ok(());
    }
    state.pending_delete = Some(if let Some(client) = scoped_org_client(args, &node)? {
        let mut scoped = |method: Method,
                          path: &str,
                          params: &[(String, String)],
                          payload: Option<&Value>|
         -> Result<Option<Value>> {
            client.request_json(method, path, params, payload)
        };
        build_delete_preview(&mut scoped, args, &node, include_folders)?
    } else {
        build_delete_preview(request_json, args, &node, include_folders)?
    });
    state.detail_scroll = 0;
    state.status = delete_status_message(&node, include_folders);
    Ok(())
}

fn confirm_delete<F>(
    request_json: &mut F,
    args: &BrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(plan) = state.pending_delete.take() else {
        return Ok(());
    };
    let Some(node) = state.selected_node().cloned() else {
        return Ok(());
    };
    let deleted = if let Some(client) = scoped_org_client(args, &node)? {
        let mut scoped = |method: Method,
                          path: &str,
                          params: &[(String, String)],
                          payload: Option<&Value>|
         -> Result<Option<Value>> {
            client.request_json(method, path, params, payload)
        };
        execute_delete_plan_with_request(&mut scoped, &plan)?
    } else {
        execute_delete_plan_with_request(request_json, &plan)?
    };
    let document = refresh_browser_document(request_json, args)?;
    state.replace_document(document);
    state.status = format!("Deleted {} item(s) from the live dashboard tree.", deleted);
    ensure_selected_dashboard_view(request_json, args, state, false)?;
    Ok(())
}

fn repeat_search<F>(request_json: &mut F, args: &BrowseArgs, state: &mut BrowserState) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let Some(search) = state.last_search.clone() else {
        state.status = "No previous dashboard search. Use / or ? first.".to_string();
        return Ok(());
    };
    if let Some(index) = state.repeat_last_search() {
        state.select_index(index);
        ensure_selected_dashboard_view(request_json, args, state, false)?;
        state.status = format!(
            "Next match for '{}' at tree row {}.",
            search.query,
            index + 1
        );
    } else {
        state.status = format!("No more matches for '{}'.", search.query);
    }
    Ok(())
}

fn live_view_cache_key(node: &DashboardBrowseNode) -> Option<String> {
    node.uid
        .as_ref()
        .map(|uid| format!("{}::{uid}", node.org_id))
}

fn scoped_org_client(
    args: &BrowseArgs,
    node: &DashboardBrowseNode,
) -> Result<Option<crate::http::JsonHttpClient>> {
    if !args.all_orgs {
        return Ok(None);
    }
    let org_id = node.org_id.parse::<i64>().map_err(|_| {
        message(format!(
            "Dashboard browse could not parse org id '{}'.",
            node.org_id
        ))
    })?;
    Ok(Some(build_http_client_for_org(&args.common, org_id)?))
}

#[cfg(test)]
mod tests {
    use super::super::browse_support::DashboardBrowseDocument;
    use super::super::browse_support::DashboardBrowseSummary;
    use super::*;

    fn empty_document() -> DashboardBrowseDocument {
        DashboardBrowseDocument {
            summary: DashboardBrowseSummary {
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
    fn search_prompt_treats_q_as_query_text() {
        let mut state = BrowserState::new(empty_document());
        state.start_search(SearchDirection::Forward);

        handle_search_dialog_key(
            &mut state,
            &KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        )
        .expect("search key should succeed");

        assert_eq!(
            state
                .pending_search
                .as_ref()
                .map(|search| search.query.as_str()),
            Some("q")
        );
    }

    #[test]
    fn external_edit_preview_lines_hide_staging_path_noise() {
        let lines = render_external_edit_preview_lines(
            "create-or-update",
            &[[
                "two-prom-query-smoke".to_string(),
                "exists".to_string(),
                "update".to_string(),
                "General".to_string(),
                "General".to_string(),
                "General".to_string(),
                "".to_string(),
                "/tmp/grafana-dashboard-browse-preview/dashboard.json".to_string(),
            ]],
        );

        let joined = lines.join("\n");
        assert!(joined.contains("Preview only. Nothing has been written to Grafana yet."));
        assert!(joined.contains("Dashboard: two-prom-query-smoke"));
        assert!(joined.contains("Action: update"));
        assert!(joined.contains("Folder: General"));
        assert!(!joined.contains("dashboard.json"));
    }
}
