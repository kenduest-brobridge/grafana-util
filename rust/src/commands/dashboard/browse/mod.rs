//! Interactive browse workflows and terminal-driven state flow for Dashboard entities.

#[cfg(feature = "tui")]
use std::io::{stdin, stdout, IsTerminal};

#[cfg(feature = "tui")]
use crate::common::message;
use crate::common::Result;
use crate::http::JsonHttpClient;

pub(crate) mod actions;
pub(crate) mod edit_dialog;
pub(crate) mod external_edit_dialog;
pub(crate) mod history_dialog;
pub(crate) mod input;
pub(crate) mod live_detail;
pub(crate) mod load;
pub(crate) mod render;
pub(crate) mod state;
pub(crate) mod support;
pub(crate) mod terminal;
pub(crate) mod tui;

pub(crate) use actions as browse_actions;
pub(crate) use edit_dialog as browse_edit_dialog;
pub(crate) use external_edit_dialog as browse_external_edit_dialog;
pub(crate) use history_dialog as browse_history_dialog;
pub(crate) use input as browse_input;
pub(crate) use render as browse_render;
pub(crate) use state as browse_state;
pub(crate) use support as browse_support;
pub(crate) use terminal as browse_terminal;

#[cfg(feature = "tui")]
use self::tui::run_dashboard_browser_tui;
use super::BrowseArgs;
#[cfg(feature = "tui")]
use super::{build_http_client, build_http_client_for_org};

pub(crate) fn uses_local_browse_source(args: &BrowseArgs) -> bool {
    args.input_dir.is_some() || args.workspace.is_some()
}

#[cfg(feature = "tui")]
pub(crate) fn browse_dashboards_with_client(
    client: &JsonHttpClient,
    args: &BrowseArgs,
) -> Result<usize> {
    if uses_local_browse_source(args) {
        return browse_dashboards_with_local_args(args);
    }
    ensure_interactive_terminal()?;
    run_dashboard_browser_tui(
        |method, path, params, payload| client.request_json(method, path, params, payload),
        args,
    )
}

#[cfg(feature = "tui")]
pub(crate) fn browse_dashboards_with_org_client(args: &BrowseArgs) -> Result<usize> {
    if uses_local_browse_source(args) {
        return browse_dashboards_with_local_args(args);
    }
    let client = if args.all_orgs {
        build_http_client(&args.common)?
    } else {
        match args.org_id {
            Some(org_id) => build_http_client_for_org(&args.common, org_id)?,
            None => build_http_client(&args.common)?,
        }
    };
    browse_dashboards_with_client(&client, args)
}

#[cfg(feature = "tui")]
fn browse_dashboards_with_local_args(args: &BrowseArgs) -> Result<usize> {
    ensure_interactive_terminal()?;
    run_dashboard_browser_tui(
        |_method, _path, _params, _payload| {
            Err(message(
                "Local dashboard browse does not use live Grafana requests.",
            ))
        },
        args,
    )
}

#[cfg(feature = "tui")]
fn ensure_interactive_terminal() -> Result<()> {
    if stdin().is_terminal() && stdout().is_terminal() {
        Ok(())
    } else {
        Err(message(
            "Dashboard browse requires an interactive terminal (TTY).",
        ))
    }
}

#[cfg(not(feature = "tui"))]
pub(crate) fn browse_dashboards_with_client(
    _client: &JsonHttpClient,
    _args: &BrowseArgs,
) -> Result<usize> {
    super::tui_not_built("browse")
}

#[cfg(not(feature = "tui"))]
pub(crate) fn browse_dashboards_with_org_client(_args: &BrowseArgs) -> Result<usize> {
    super::tui_not_built("browse")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::CliColorChoice;
    use crate::dashboard::{CommonCliArgs, DashboardImportInputFormat};
    use std::path::PathBuf;

    fn make_browse_args() -> BrowseArgs {
        BrowseArgs {
            common: CommonCliArgs {
                color: CliColorChoice::Auto,
                profile: None,
                url: "https://grafana.example.com".to_string(),
                api_token: Some("secret".to_string()),
                username: None,
                password: None,
                prompt_password: false,
                prompt_token: false,
                timeout: 30,
                verify_ssl: false,
            },
            workspace: None,
            input_dir: None,
            local: false,
            run: None,
            run_id: None,
            input_format: DashboardImportInputFormat::Raw,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            path: None,
        }
    }

    #[test]
    fn workspace_roots_are_treated_as_local_browse_sources() {
        let mut args = make_browse_args();
        assert!(!uses_local_browse_source(&args));

        args.workspace = Some(PathBuf::from("/tmp/git-sync-workspace"));
        assert!(uses_local_browse_source(&args));

        args.workspace = None;
        args.input_dir = Some(PathBuf::from("/tmp/raw"));
        assert!(uses_local_browse_source(&args));
    }
}
