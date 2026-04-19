#![cfg(feature = "tui")]
use crossterm::event::KeyEvent;
use reqwest::Method;
use serde_json::Value;
use std::fs;

use crate::common::{message, Result};

use super::browse_external_edit_workflow::preview_external_edit_dry_run;
pub(super) use super::browse_external_edit_workflow::run_selected_external_edit;
use super::browse_input_shared::{redraw_browser, scoped_org_client};
use crate::dashboard::browse_actions::{apply_external_dashboard_edit, refresh_browser_document};
use crate::dashboard::browse_external_edit_dialog::{
    ExternalEditDialogAction, ExternalEditErrorAction,
};
use crate::dashboard::browse_state::{BrowserState, CompletionNotice};
use crate::dashboard::browse_terminal::TerminalSession;
use crate::dashboard::BrowseArgs;

pub(super) fn handle_external_edit_error_key<F>(
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
        .pending_external_edit_error
        .as_ref()
        .map(|dialog| dialog.handle_key(key))
    else {
        return Ok(());
    };
    match action {
        ExternalEditErrorAction::Continue => {}
        ExternalEditErrorAction::Close => {
            let uid = state
                .pending_external_edit_error
                .as_ref()
                .map(|dialog| dialog.uid.clone())
                .unwrap_or_else(|| "dashboard".to_string());
            state.pending_external_edit_error = None;
            state.status = format!("Aborted raw JSON edit for {}.", uid);
        }
        ExternalEditErrorAction::Retry => {
            state.pending_external_edit_error = None;
            run_selected_external_edit(request_json, args, session, state)?;
        }
    }
    Ok(())
}

pub(super) fn handle_external_edit_dialog_key<F>(
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
            super::ensure_selected_dashboard_view(request_json, args, state, false)?;
        }
    }
    Ok(())
}
