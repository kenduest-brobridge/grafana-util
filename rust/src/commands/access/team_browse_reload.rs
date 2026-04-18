//! Team browser refresh and reload behavior.

use reqwest::Method;
use serde_json::Value;

use crate::access::TeamBrowseArgs;
use crate::common::Result;

use super::team_browse_input::load_rows;
use super::team_browse_state::BrowserState;

pub(super) fn reload_rows<F>(
    request_json: &mut F,
    args: &TeamBrowseArgs,
    state: &mut BrowserState,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    state.replace_rows(load_rows(request_json, args)?);
    state.status = if args.input_dir.is_some() {
        "Reloaded team browser from local bundle.".to_string()
    } else {
        "Refreshed team browser from live Grafana.".to_string()
    };
    Ok(())
}
