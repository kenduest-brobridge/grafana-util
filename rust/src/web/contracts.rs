use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::common::GrafanaCliError;

#[derive(Debug, Clone, Default)]
pub(crate) struct WebState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FieldOptionDescriptor {
    pub(crate) value: &'static str,
    pub(crate) label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionFieldDescriptor {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) required: bool,
    pub(crate) help: &'static str,
    pub(crate) placeholder: &'static str,
    pub(crate) default_value: Value,
    pub(crate) options: Vec<FieldOptionDescriptor>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionDescriptor {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) ui_mode: &'static str,
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) read_only: bool,
    pub(crate) requires_connection: bool,
    pub(crate) example_request: Value,
    pub(crate) fields: Vec<ActionFieldDescriptor>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceDescriptor {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) actions: Vec<ActionDescriptor>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CapabilityDescriptor {
    pub(crate) id: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) read_only: bool,
    pub(crate) dry_run_supported: bool,
    pub(crate) live_apply_supported: bool,
    pub(crate) example_request: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionRunResponse {
    pub(crate) action: &'static str,
    pub(crate) title: &'static str,
    pub(crate) ui_mode: &'static str,
    pub(crate) read_only: bool,
    pub(crate) text_lines: Vec<String>,
    pub(crate) document: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectionTestResponse {
    pub(crate) ok: bool,
    pub(crate) url: String,
    pub(crate) auth_mode: String,
    pub(crate) org_id: Option<i64>,
    pub(crate) org_name: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug)]
pub(crate) struct WebError {
    pub(crate) status: StatusCode,
    pub(crate) message: String,
    pub(crate) kind: &'static str,
}

impl From<GrafanaCliError> for WebError {
    fn from(error: GrafanaCliError) -> Self {
        let kind = error.kind();
        let status = match kind {
            "message" | "validation" | "url" | "header-name" | "header-value" | "parse"
            | "json" => StatusCode::BAD_REQUEST,
            "api-response" | "http" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            message: error.to_string(),
            kind,
        }
    }
}

impl From<serde_json::Error> for WebError {
    fn from(error: serde_json::Error) -> Self {
        WebError::from(GrafanaCliError::from(error))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": self.message,
                "kind": self.kind,
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommonConnectionRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardBrowseRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) page_size: Option<usize>,
    pub(crate) org_id: Option<i64>,
    #[serde(default)]
    pub(crate) all_orgs: bool,
    #[serde(default)]
    pub(crate) with_sources: bool,
    pub(crate) query: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) selected_uid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardListRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) page_size: Option<usize>,
    pub(crate) org_id: Option<i64>,
    #[serde(default)]
    pub(crate) all_orgs: bool,
    #[serde(default)]
    pub(crate) with_sources: bool,
    pub(crate) output_columns: Option<String>,
    pub(crate) output_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardInspectLiveRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) page_size: Option<usize>,
    pub(crate) concurrency: Option<usize>,
    pub(crate) org_id: Option<i64>,
    #[serde(default)]
    pub(crate) all_orgs: bool,
    pub(crate) output_mode: Option<String>,
    pub(crate) report_columns: Option<String>,
    pub(crate) report_filter_datasource: Option<String>,
    pub(crate) report_filter_panel_id: Option<String>,
    #[serde(default)]
    pub(crate) progress: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardInspectExportRequest {
    pub(crate) import_dir: String,
    pub(crate) output_mode: Option<String>,
    pub(crate) report_columns: Option<String>,
    pub(crate) report_filter_datasource: Option<String>,
    pub(crate) report_filter_panel_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DashboardInspectVarsRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) dashboard_uid: Option<String>,
    pub(crate) dashboard_url: Option<String>,
    pub(crate) vars_query: Option<String>,
    pub(crate) org_id: Option<i64>,
    pub(crate) output_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OverviewStagedRequest {
    pub(crate) dashboard_export_dir: Option<String>,
    pub(crate) datasource_export_dir: Option<String>,
    pub(crate) access_user_export_dir: Option<String>,
    pub(crate) access_team_export_dir: Option<String>,
    pub(crate) access_org_export_dir: Option<String>,
    pub(crate) access_service_account_export_dir: Option<String>,
    pub(crate) desired_file: Option<String>,
    pub(crate) source_bundle: Option<String>,
    pub(crate) target_inventory: Option<String>,
    pub(crate) alert_export_dir: Option<String>,
    pub(crate) availability_file: Option<String>,
    pub(crate) mapping_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectStatusStagedRequest {
    pub(crate) dashboard_export_dir: Option<String>,
    pub(crate) datasource_export_dir: Option<String>,
    pub(crate) access_user_export_dir: Option<String>,
    pub(crate) access_team_export_dir: Option<String>,
    pub(crate) access_org_export_dir: Option<String>,
    pub(crate) access_service_account_export_dir: Option<String>,
    pub(crate) desired_file: Option<String>,
    pub(crate) source_bundle: Option<String>,
    pub(crate) target_inventory: Option<String>,
    pub(crate) alert_export_dir: Option<String>,
    pub(crate) availability_file: Option<String>,
    pub(crate) mapping_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectStatusLiveRequest {
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    #[serde(default)]
    pub(crate) insecure: bool,
    pub(crate) ca_cert: Option<String>,
    #[serde(default)]
    pub(crate) all_orgs: bool,
    pub(crate) org_id: Option<i64>,
    pub(crate) sync_summary_file: Option<String>,
    pub(crate) bundle_preflight_file: Option<String>,
    pub(crate) promotion_summary_file: Option<String>,
    pub(crate) mapping_file: Option<String>,
    pub(crate) availability_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncSummaryRequest {
    pub(crate) desired_file: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncPlanRequest {
    pub(crate) desired_file: String,
    pub(crate) live_file: Option<String>,
    #[serde(default)]
    pub(crate) fetch_live: bool,
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) org_id: Option<i64>,
    pub(crate) page_size: Option<usize>,
    #[serde(default)]
    pub(crate) allow_prune: bool,
    pub(crate) trace_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncPreflightRequest {
    pub(crate) desired_file: String,
    pub(crate) availability_file: Option<String>,
    #[serde(default)]
    pub(crate) fetch_live: bool,
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) org_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncBundlePreflightRequest {
    pub(crate) source_bundle: String,
    pub(crate) target_inventory: String,
    pub(crate) availability_file: Option<String>,
    #[serde(default)]
    pub(crate) fetch_live: bool,
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) org_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncPromotionPreflightRequest {
    pub(crate) source_bundle: String,
    pub(crate) target_inventory: String,
    pub(crate) mapping_file: Option<String>,
    pub(crate) availability_file: Option<String>,
    #[serde(default)]
    pub(crate) fetch_live: bool,
    pub(crate) url: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) timeout: Option<u64>,
    #[serde(default)]
    pub(crate) verify_ssl: bool,
    pub(crate) org_id: Option<i64>,
}
