//! Local-only web workbench over shared Rust execution contracts.
//!
//! The workbench is intentionally localhost-first and keeps credentials browser-session scoped.

mod contracts;
mod handlers;
mod page;
mod registry;

use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use clap::Parser;
use std::net::{IpAddr, SocketAddr};

use crate::common::{message, Result};

use self::contracts::WebState;
use self::handlers::{
    capabilities_handler, index_handler, run_dashboard_browse_handler,
    run_dashboard_inspect_export_handler, run_dashboard_inspect_live_handler,
    run_dashboard_inspect_vars_handler, run_dashboard_list_handler, run_overview_staged_handler,
    run_project_status_live_handler, run_project_status_staged_handler,
    run_sync_bundle_preflight_handler, run_sync_plan_handler, run_sync_preflight_handler,
    run_sync_promotion_preflight_handler, run_sync_summary_handler, test_connection_handler,
    workspaces_handler,
};

const INDEX_CSS: &str = include_str!("index.css");
const INDEX_JS: &str = include_str!("index.js");
const SHELL_STATE_JS: &str = include_str!("shell_state.js");
const SHELL_UI_JS: &str = include_str!("shell_ui.js");
const CONNECTION_UI_JS: &str = include_str!("connection_ui.js");
const PALETTE_JS: &str = include_str!("palette.js");
const RESULT_STAGE_JS: &str = include_str!("result_stage.js");
const RENDERERS_JS: &str = include_str!("renderers.js");

#[derive(Debug, Clone, Parser)]
#[command(
    name = "grafana-util-web",
    about = "Run the local-only Grafana Utils web workbench."
)]
pub struct WebServerArgs {
    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Listen address. Keep this local-only unless you are explicitly proxying it."
    )]
    pub host: String,
    #[arg(long, default_value_t = 7788, help = "Listen port.")]
    pub port: u16,
}

impl WebServerArgs {
    fn socket_addr(&self) -> Result<SocketAddr> {
        let ip = self
            .host
            .parse::<IpAddr>()
            .map_err(|error| message(format!("Invalid --host value {:?}: {}", self.host, error)))?;
        Ok(SocketAddr::from((ip, self.port)))
    }
}

async fn index_css_handler() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        INDEX_CSS,
    )
}

fn javascript_asset_response(source: &'static str) -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/javascript; charset=utf-8"),
        )],
        source,
    )
        .into_response()
}

async fn asset_js_handler(Path(asset): Path<String>) -> Response {
    match asset.as_str() {
        "index.js" => javascript_asset_response(INDEX_JS),
        "shell_state.js" => javascript_asset_response(SHELL_STATE_JS),
        "shell_ui.js" => javascript_asset_response(SHELL_UI_JS),
        "connection_ui.js" => javascript_asset_response(CONNECTION_UI_JS),
        "palette.js" => javascript_asset_response(PALETTE_JS),
        "result_stage.js" => javascript_asset_response(RESULT_STAGE_JS),
        "renderers.js" => javascript_asset_response(RENDERERS_JS),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn build_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/assets/index.css", get(index_css_handler))
        .route("/assets/{asset}", get(asset_js_handler))
        .route("/api/1.0/workspaces", get(workspaces_handler))
        .route("/api/1.0/capabilities", get(capabilities_handler))
        .route("/api/1.0/connection/test", post(test_connection_handler))
        .route(
            "/api/1.0/dashboard/browse",
            post(run_dashboard_browse_handler),
        )
        .route("/api/1.0/dashboard/list", post(run_dashboard_list_handler))
        .route(
            "/api/1.0/dashboard/inspect-live",
            post(run_dashboard_inspect_live_handler),
        )
        .route(
            "/api/1.0/dashboard/inspect-export",
            post(run_dashboard_inspect_export_handler),
        )
        .route(
            "/api/1.0/dashboard/inspect-vars",
            post(run_dashboard_inspect_vars_handler),
        )
        .route(
            "/api/1.0/overview/staged",
            post(run_overview_staged_handler),
        )
        .route(
            "/api/1.0/project-status/staged",
            post(run_project_status_staged_handler),
        )
        .route(
            "/api/1.0/project-status/live",
            post(run_project_status_live_handler),
        )
        .route("/api/1.0/sync/summary", post(run_sync_summary_handler))
        .route("/api/1.0/sync/plan", post(run_sync_plan_handler))
        .route("/api/1.0/sync/preflight", post(run_sync_preflight_handler))
        .route(
            "/api/1.0/sync/bundle-preflight",
            post(run_sync_bundle_preflight_handler),
        )
        .route(
            "/api/1.0/sync/promotion-preflight",
            post(run_sync_promotion_preflight_handler),
        )
        .with_state(WebState)
}

/// Run the local web workbench until the server is stopped.
pub async fn run_web_server(args: WebServerArgs) -> Result<()> {
    let address = args.socket_addr()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Grafana Utils web workbench listening on http://{address}");
    axum::serve(listener, build_router()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_router;
    use super::contracts::CommonConnectionRequest;
    use super::handlers::{build_dashboard_browse_document, test_connection};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;
    use tower::util::ServiceExt;

    fn spawn_connection_test_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();

            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let bytes_read = stream.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            assert!(request_text.starts_with("GET /api/org HTTP/1.1"));

            let body = r#"{"id":7,"name":"Test Org"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            let _ = stream.flush();
        });
        (format!("http://{address}"), handle)
    }

    #[tokio::test]
    async fn workspace_registry_lists_dashboard_and_sync_domains() {
        let app = build_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/1.0/workspaces")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        let ids = value
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item.get("id").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(ids.contains(&"dashboard"));
        assert!(ids.contains(&"sync"));
    }

    #[tokio::test]
    async fn legacy_workspace_registry_alias_is_not_registered() {
        let app = build_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/workspaces")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn root_html_mounts_embedded_assets() {
        let app = build_router();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("id=\"app\""));
        assert!(html.contains("href=\"/assets/index.css\""));
        assert!(html.contains("src=\"/assets/index.js\""));
    }

    #[test]
    fn dashboard_browse_document_groups_rows_and_selects_detail() {
        let rows = vec![
            serde_json::from_value(json!({
                "uid": "cpu-main",
                "title": "CPU Main",
                "folderTitle": "Infra",
                "folderUid": "infra",
                "folderPath": "Platform / Infra",
                "orgName": "Main Org.",
                "orgId": 1,
                "sources": ["Prometheus Main"]
            }))
            .unwrap(),
            serde_json::from_value(json!({
                "uid": "logs-main",
                "title": "Logs Main",
                "folderTitle": "Observability",
                "folderUid": "obs",
                "folderPath": "Platform / Observability",
                "orgName": "Main Org.",
                "orgId": 1,
                "sources": ["Loki Main"]
            }))
            .unwrap(),
        ];
        let document = build_dashboard_browse_document(&rows, Some("cpu"), None, Some("cpu-main"));
        assert_eq!(document["kind"], "grafana-utils-dashboard-browser");
        assert_eq!(document["summary"]["dashboardCount"], 1);
        assert_eq!(document["detail"]["uid"], "cpu-main");
        assert_eq!(
            document["tree"][0]["children"][0]["folderPath"],
            "Platform / Infra"
        );
    }

    #[tokio::test]
    async fn sync_summary_endpoint_returns_shared_document_and_text_preview() {
        let temp = tempdir().unwrap();
        let desired_file = temp.path().join("desired.json");
        fs::write(
            &desired_file,
            serde_json::to_string_pretty(&json!([
                {"kind":"dashboard","identity":"cpu-main","title":"CPU Main","body":{"uid":"cpu-main"}}
            ]))
            .unwrap(),
        )
        .unwrap();
        let app = build_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/1.0/sync/summary")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"desiredFile": desired_file.display().to_string()}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["action"], "sync-summary");
        assert_eq!(value["document"]["kind"], "grafana-utils-sync-summary");
        assert!(!value["textLines"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn overview_staged_endpoint_reuses_existing_overview_document_builder() {
        let temp = tempdir().unwrap();
        let desired_file = temp.path().join("desired.json");
        fs::write(&desired_file, "[]").unwrap();
        let app = build_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/1.0/overview/staged")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"desiredFile": desired_file.display().to_string()}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["action"], "overview-staged");
        assert_eq!(value["document"]["kind"], "grafana-utils-overview");
        assert!(value["document"]["toolVersion"].is_string());
    }

    #[test]
    fn connection_test_probe_returns_org_identity() {
        let (base_url, handle) = spawn_connection_test_server();
        let response = test_connection(&CommonConnectionRequest {
            url: Some(base_url),
            api_token: None,
            username: Some("admin".to_string()),
            password: Some("admin".to_string()),
            timeout: Some(30),
            verify_ssl: false,
        })
        .unwrap();
        handle.join().unwrap();
        assert!(response.ok);
        assert_eq!(response.auth_mode, "basic");
        assert_eq!(response.org_id, Some(7));
        assert_eq!(response.org_name.as_deref(), Some("Test Org"));
    }
}
