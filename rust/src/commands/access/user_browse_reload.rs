//! User browser refresh and reload behavior.

use reqwest::Method;
use serde_json::Value;

use crate::access::{Result, UserBrowseArgs};

use super::user_browse_input::load_rows;
use super::user_browse_state::BrowserState;

pub(super) fn reload_rows<F>(
    request_json: &mut F,
    args: &UserBrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    state.replace_rows(load_rows(request_json, args, state.display_mode)?);
    state.status = if args.input_dir.is_some() {
        "Reloaded user browser from local bundle.".to_string()
    } else {
        "Refreshed user browser from live Grafana.".to_string()
    };
    Ok(())
}
