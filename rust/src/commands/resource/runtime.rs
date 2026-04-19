use reqwest::Method;
use serde_json::{Map, Value};

use super::catalog::ResourceSelector;
use super::cli_defs::{ResourceCliArgs, ResourceCommand, ResourceKind};
use super::render::{render_describe, render_get, render_kind_catalog, render_list};
use crate::common::{set_json_color_choice, Result};
use crate::dashboard::{CommonCliArgs, DEFAULT_TIMEOUT, DEFAULT_URL};
use crate::grafana_api::{
    expect_object, expect_object_list, AuthInputs, GrafanaApiClient, GrafanaConnection,
};
use crate::profile_config::ConnectionMergeInput;

pub(crate) fn build_client(common: &CommonCliArgs) -> Result<GrafanaApiClient> {
    let connection = GrafanaConnection::resolve(
        common.profile.as_deref(),
        ConnectionMergeInput {
            url: &common.url,
            url_default: DEFAULT_URL,
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            org_id: None,
            timeout: common.timeout,
            timeout_default: DEFAULT_TIMEOUT,
            verify_ssl: common.verify_ssl,
            insecure: false,
            ca_cert: None,
        },
        AuthInputs {
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            prompt_password: common.prompt_password,
            prompt_token: common.prompt_token,
        },
        false,
    )?;
    GrafanaApiClient::from_connection(connection)
}

pub(crate) fn list_resource_items(
    client: &GrafanaApiClient,
    kind: ResourceKind,
) -> Result<Vec<Map<String, Value>>> {
    match kind {
        ResourceKind::Dashboards => client.dashboard().list_dashboard_summaries(500),
        ResourceKind::Folders => expect_object_list(
            client
                .dashboard()
                .request_json(Method::GET, "/api/folders", &[], None)?,
            "Unexpected folder list response from Grafana.",
        ),
        ResourceKind::Datasources => client.datasource().list_datasources(),
        ResourceKind::AlertRules => client.alerting().list_alert_rules(),
        ResourceKind::Orgs => client.access().list_orgs(),
    }
}

pub(crate) fn get_resource_item(
    client: &GrafanaApiClient,
    selector: &ResourceSelector,
) -> Result<Value> {
    match selector.kind {
        ResourceKind::Dashboards => client.dashboard().fetch_dashboard(&selector.identity),
        ResourceKind::Folders => Ok(Value::Object(expect_object(
            client.dashboard().request_json(
                Method::GET,
                &format!("/api/folders/{}", selector.identity),
                &[],
                None,
            )?,
            "Unexpected folder payload from Grafana.",
        )?)),
        ResourceKind::Datasources => Ok(Value::Object(expect_object(
            client.datasource().request_json(
                Method::GET,
                &format!("/api/datasources/uid/{}", selector.identity),
                &[],
                None,
            )?,
            "Unexpected datasource payload from Grafana.",
        )?)),
        ResourceKind::AlertRules => Ok(Value::Object(
            client.alerting().get_alert_rule(&selector.identity)?,
        )),
        ResourceKind::Orgs => Ok(Value::Object(expect_object(
            client.access().request_json(
                Method::GET,
                &format!("/api/orgs/{}", selector.identity),
                &[],
                None,
            )?,
            "Unexpected org payload from Grafana.",
        )?)),
    }
}

pub fn run_resource_cli(args: ResourceCliArgs) -> Result<()> {
    // Resource is a pure read-only surface; dispatch only routes to renderers,
    // with output shape determined by each command variant.
    set_json_color_choice(args.color);
    match args.command {
        ResourceCommand::Kinds(inner) => render_kind_catalog(&inner),
        ResourceCommand::Describe(inner) => render_describe(&inner),
        ResourceCommand::List(inner) => render_list(&inner),
        ResourceCommand::Get(inner) => render_get(&inner),
    }
}
