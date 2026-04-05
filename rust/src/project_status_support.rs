//! Shared project-status helpers used by live status workflows.
//!
//! Responsibilities:
//! - Build authenticated HTTP clients and auth header sets for status checks.
//! - Resolve per-org connection settings and default behavior for live runs.

use crate::common::Result as CommonResult;
use crate::grafana_api::{AuthInputs, GrafanaApiClient, GrafanaConnection};
use crate::http::JsonHttpClient;
use crate::profile_config::ConnectionMergeInput;
use crate::project_status_command::ProjectStatusLiveArgs;

fn resolve_live_project_status_connection(
    args: &ProjectStatusLiveArgs,
    org_id: Option<i64>,
    include_org_header: bool,
) -> CommonResult<GrafanaConnection> {
    GrafanaConnection::resolve(
        args.profile.as_deref(),
        ConnectionMergeInput {
            url: &args.url,
            url_default: "http://localhost:3000",
            api_token: args.api_token.as_deref(),
            username: args.username.as_deref(),
            password: args.password.as_deref(),
            org_id,
            timeout: args.timeout,
            timeout_default: 30,
            verify_ssl: args.verify_ssl,
            insecure: args.insecure,
            ca_cert: args.ca_cert.as_deref(),
        },
        AuthInputs {
            api_token: args.api_token.as_deref(),
            username: args.username.as_deref(),
            password: args.password.as_deref(),
            prompt_password: args.prompt_password,
            prompt_token: args.prompt_token,
        },
        include_org_header,
    )
}

#[cfg(test)]
pub(crate) fn resolve_live_project_status_headers(
    args: &ProjectStatusLiveArgs,
    org_id: Option<i64>,
) -> CommonResult<Vec<(String, String)>> {
    let connection = resolve_live_project_status_connection(args, org_id, true)?;
    Ok(connection.headers)
}

pub(crate) fn build_live_project_status_client(
    args: &ProjectStatusLiveArgs,
) -> CommonResult<JsonHttpClient> {
    build_live_project_status_client_for_org(args, if args.all_orgs { None } else { args.org_id })
}

pub(crate) fn build_live_project_status_client_for_org(
    args: &ProjectStatusLiveArgs,
    org_id: Option<i64>,
) -> CommonResult<JsonHttpClient> {
    let connection = resolve_live_project_status_connection(args, org_id, true)?;
    Ok(GrafanaApiClient::from_connection(connection)?.into_http_client())
}

#[cfg(test)]
mod tests {
    use super::resolve_live_project_status_headers;
    use crate::project_status_command::{ProjectStatusLiveArgs, ProjectStatusOutputFormat};

    #[test]
    fn resolve_live_project_status_headers_adds_org_scope_when_requested() {
        let args = ProjectStatusLiveArgs {
            profile: None,
            url: "http://localhost:3000".to_string(),
            api_token: Some("token-123".to_string()),
            username: None,
            password: None,
            prompt_password: false,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
            insecure: false,
            ca_cert: None,
            all_orgs: false,
            org_id: Some(7),
            sync_summary_file: None,
            bundle_preflight_file: None,
            promotion_summary_file: None,
            mapping_file: None,
            availability_file: None,
            output_format: ProjectStatusOutputFormat::Text,
        };

        let headers = resolve_live_project_status_headers(&args, args.org_id).unwrap();

        assert!(headers
            .iter()
            .any(|(name, value)| { name == "X-Grafana-Org-Id" && value == "7" }));
    }
}
