use super::live_discovery::build_live_instance_discovery_with_request;
use super::live_domains::{
    build_live_dashboard_and_datasource_statuses, build_live_promotion_status,
    build_live_sync_status,
};
use super::live_multi_org::{
    build_live_multi_org_domain_status_pair, build_live_multi_org_domain_status_with_clients,
    build_live_multi_org_domain_status_with_orgs, scoped_live_org_client_for_test,
};
use crate::http::{JsonHttpClient, JsonHttpClientConfig};
use crate::project_status::{
    status_finding, ProjectDomainStatus, ProjectStatusFreshness, PROJECT_STATUS_BLOCKED,
    PROJECT_STATUS_PARTIAL, PROJECT_STATUS_READY, PROJECT_STATUS_UNKNOWN,
};
use crate::project_status_freshness::build_live_project_status_freshness_from_samples;
use crate::project_status_support::project_status_live;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

const TEST_DASHBOARD_LIMIT: &str = "500";

#[test]
fn build_live_instance_discovery_records_api_health_payload() {
    let mut calls = 0usize;
    let discovery = build_live_instance_discovery_with_request(
        &mut |method, path, params: &[(String, String)], payload| {
            calls += 1;
            assert_eq!(method, Method::GET);
            assert_eq!(path, "/api/health");
            assert!(params.is_empty());
            assert!(payload.is_none());
            Ok(Some(json!({
                "database": "ok",
                "version": "12.4.0",
                "commit": "abc123"
            })))
        },
    );

    assert_eq!(calls, 1);
    assert_eq!(discovery["source"], "api-health");
    assert_eq!(discovery["status"], "available");
    assert_eq!(discovery["health"]["database"], "ok");
    assert_eq!(discovery["health"]["version"], "12.4.0");
    assert_eq!(discovery["health"]["commit"], "abc123");
}

#[test]
fn build_live_instance_discovery_records_health_failure_without_erroring() {
    let discovery =
        build_live_instance_discovery_with_request(&mut |_method, _path, _params, _payload| {
            Err(crate::common::message("health read failed"))
        });

    assert_eq!(discovery["source"], "api-health");
    assert_eq!(discovery["status"], "unavailable");
    assert!(discovery["error"]
        .as_str()
        .unwrap()
        .contains("health read failed"));
    assert!(discovery.get("health").is_none());
}

#[test]
fn build_live_sync_status_uses_staged_input_metadata_for_freshness() {
    let dir = tempdir().unwrap();
    let summary_path = dir.path().join("sync-summary.json");
    let bundle_path = dir.path().join("bundle-preflight.json");
    fs::write(&summary_path, "{}").unwrap();
    fs::write(
        &bundle_path,
        r#"{"summary":{"resourceCount":1,"syncBlockingCount":0}}"#,
    )
    .unwrap();
    let summary_metadata = fs::metadata(&summary_path).unwrap();
    let bundle_metadata = fs::metadata(&bundle_path).unwrap();
    let summary_document = json!({"summary":{"resourceCount":1}});
    let bundle_document = json!({"summary":{"resourceCount":1,"syncBlockingCount":0}});

    let status = build_live_sync_status(
        Some(&summary_document),
        Some(&bundle_document),
        Some(&summary_metadata),
        Some(&bundle_metadata),
    );

    assert_eq!(status.freshness.status, "current");
    assert_eq!(status.freshness.source_count, 2);
    assert!(status.freshness.newest_age_seconds.is_some());
    assert!(status.freshness.oldest_age_seconds.is_some());
}

#[test]
fn build_live_promotion_status_uses_staged_input_metadata_for_freshness() {
    let dir = tempdir().unwrap();
    let summary_path = dir.path().join("promotion-summary.json");
    let mapping_path = dir.path().join("mapping.json");
    let availability_path = dir.path().join("availability.json");
    fs::write(
        &summary_path,
        r#"{"summary":{"resourceCount":1,"blockingCount":0},"handoffSummary":{"readyForReview":false}}"#,
    )
    .unwrap();
    fs::write(&mapping_path, "{}").unwrap();
    fs::write(&availability_path, "{}").unwrap();
    let summary_metadata = fs::metadata(&summary_path).unwrap();
    let mapping_metadata = fs::metadata(&mapping_path).unwrap();
    let availability_metadata = fs::metadata(&availability_path).unwrap();
    let summary_document = json!({"summary":{"resourceCount":1,"blockingCount":0},"handoffSummary":{"readyForReview":false}});
    let mapping_document = json!({});
    let availability_document = json!({});

    let status = build_live_promotion_status(
        Some(&summary_document),
        Some(&mapping_document),
        Some(&availability_document),
        Some(&summary_metadata),
        Some(&mapping_metadata),
        Some(&availability_metadata),
    );

    assert_eq!(status.status, PROJECT_STATUS_PARTIAL);
    assert_eq!(status.freshness.status, "current");
    assert_eq!(status.freshness.source_count, 3);
    assert!(status.freshness.newest_age_seconds.is_some());
    assert!(status.freshness.oldest_age_seconds.is_some());
}

#[test]
fn build_live_dashboard_status_uses_dashboard_version_history_for_freshness() {
    let created = DateTime::<Utc>::from(SystemTime::now() - Duration::from_secs(60)).to_rfc3339();
    let status = project_status_live::build_live_dashboard_status_with_request(
        |method, path, params: &[(String, String)], _payload| match (method, path) {
            (Method::GET, "/api/search") => {
                assert!(params
                    .iter()
                    .any(|(key, value)| key == "type" && value == "dash-db"));
                assert!(params
                    .iter()
                    .any(|(key, value)| key == "limit" && value == TEST_DASHBOARD_LIMIT));
                Ok(Some(json!([
                    {
                        "uid": "cpu-main",
                        "title": "CPU Main",
                        "type": "dash-db",
                        "folderUid": "infra",
                        "folderTitle": "Infra"
                    }
                ])))
            }
            (Method::GET, "/api/datasources") => Ok(Some(json!([
                {
                    "uid": "prom-main",
                    "name": "Prometheus Main",
                    "type": "prometheus"
                }
            ]))),
            (Method::GET, "/api/dashboards/uid/cpu-main/versions") => {
                assert_eq!(params, &vec![("limit".to_string(), "1".to_string())]);
                Ok(Some(json!([
                    {
                        "version": 7,
                        "created": created,
                        "createdBy": "admin"
                    }
                ])))
            }
            _ => Err(crate::common::message(format!("unexpected request {path}"))),
        },
    );

    assert_eq!(status.status, "ready");
    assert_eq!(status.freshness.status, "current");
    assert_eq!(status.freshness.source_count, 1);
    assert!(status.freshness.newest_age_seconds.is_some());
    assert!(status.freshness.oldest_age_seconds.is_some());
}

#[test]
fn project_status_freshness_samples_from_value_uses_timestamp_fields_from_arrays_and_objects() {
    let now = SystemTime::now();
    let updated_at = DateTime::<Utc>::from(now - Duration::from_secs(60)).to_rfc3339();
    let created_at = DateTime::<Utc>::from(now - Duration::from_secs(120)).to_rfc3339();
    let document = json!([
        {
            "uid": "rule-1",
            "updated": updated_at
        },
        {
            "uid": "rule-2",
            "created": created_at
        }
    ]);

    let samples =
        project_status_live::project_status_freshness_samples_from_value("alert-rules", &document);
    let freshness = build_live_project_status_freshness_from_samples(&samples);

    assert_eq!(samples.len(), 2);
    assert_eq!(freshness.status, "current");
    assert_eq!(freshness.source_count, 1);
    assert!(freshness.newest_age_seconds.is_some());
    assert!(freshness.oldest_age_seconds.is_some());
}

#[test]
fn build_live_multi_org_domain_status_with_orgs_fans_out_and_aggregates_counts() {
    let orgs = vec![
        json!({"id": 11, "name": "Core"})
            .as_object()
            .unwrap()
            .clone(),
        json!({"id": 22, "name": "Edge"})
            .as_object()
            .unwrap()
            .clone(),
    ];
    let mut seen_org_ids = Vec::new();

    let aggregated = build_live_multi_org_domain_status_with_orgs(&orgs, |org_id| {
        seen_org_ids.push(org_id);
        Ok(ProjectDomainStatus {
            id: "alert".to_string(),
            scope: "live".to_string(),
            mode: "live-alert-surfaces".to_string(),
            status: if org_id == 11 {
                PROJECT_STATUS_READY.to_string()
            } else {
                PROJECT_STATUS_BLOCKED.to_string()
            },
            reason_code: if org_id == 11 {
                PROJECT_STATUS_READY.to_string()
            } else {
                "blocked-by-blockers".to_string()
            },
            primary_count: if org_id == 11 { 3 } else { 5 },
            blocker_count: if org_id == 11 { 0 } else { 2 },
            warning_count: if org_id == 11 { 1 } else { 4 },
            source_kinds: vec!["alert".to_string()],
            signal_keys: vec![
                "live.alertRuleCount".to_string(),
                "live.policyCount".to_string(),
            ],
            blockers: if org_id == 11 {
                Vec::new()
            } else {
                vec![status_finding(
                    "missing-alert-policy",
                    2,
                    "live.policyCount",
                )]
            },
            warnings: vec![status_finding(
                "missing-panel-links",
                if org_id == 11 { 1 } else { 4 },
                "live.rulePanelMissingCount",
            )],
            next_actions: vec!["re-run alert checks".to_string()],
            freshness: ProjectStatusFreshness {
                status: "current".to_string(),
                source_count: 1,
                newest_age_seconds: Some(if org_id == 11 { 15 } else { 40 }),
                oldest_age_seconds: Some(if org_id == 11 { 30 } else { 55 }),
            },
        })
    })
    .unwrap();

    assert_eq!(seen_org_ids, vec![11, 22]);
    assert_eq!(aggregated.id, "alert");
    assert_eq!(aggregated.status, PROJECT_STATUS_BLOCKED);
    assert_eq!(aggregated.reason_code, "multi-org-aggregate");
    assert_eq!(aggregated.primary_count, 8);
    assert_eq!(aggregated.blocker_count, 2);
    assert_eq!(aggregated.warning_count, 5);
    assert_eq!(
        aggregated.blockers,
        vec![status_finding(
            "missing-alert-policy",
            2,
            "live.policyCount"
        )]
    );
    assert_eq!(
        aggregated.warnings,
        vec![status_finding(
            "missing-panel-links",
            5,
            "live.rulePanelMissingCount"
        )]
    );
    assert_eq!(
        aggregated.next_actions,
        vec!["re-run alert checks".to_string()]
    );
    assert_eq!(aggregated.freshness.status, "current");
    assert_eq!(aggregated.freshness.source_count, 2);
    assert_eq!(aggregated.freshness.newest_age_seconds, Some(15));
    assert_eq!(aggregated.freshness.oldest_age_seconds, Some(55));
}

#[test]
fn build_live_multi_org_domain_status_with_orgs_rejects_empty_org_lists() {
    let error = build_live_multi_org_domain_status_with_orgs(&[], |_org_id| {
        Ok(ProjectDomainStatus {
            id: "dashboard".to_string(),
            scope: "live".to_string(),
            mode: "live-dashboard-read".to_string(),
            status: PROJECT_STATUS_UNKNOWN.to_string(),
            reason_code: "unknown".to_string(),
            primary_count: 0,
            blocker_count: 0,
            warning_count: 0,
            source_kinds: Vec::new(),
            signal_keys: Vec::new(),
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_actions: Vec::new(),
            freshness: ProjectStatusFreshness::default(),
        })
    })
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("at least one per-org domain status"));
}

#[test]
fn build_live_multi_org_domain_status_with_clients_reuses_prebuilt_scoped_clients() {
    let clients = vec![
        scoped_live_org_client_for_test(11, test_http_client()),
        scoped_live_org_client_for_test(22, test_http_client()),
    ];
    let mut seen_org_ids = Vec::new();

    let aggregated = build_live_multi_org_domain_status_with_clients(&clients, |client| {
        let org_id = client.org_id();
        seen_org_ids.push(org_id);
        ProjectDomainStatus {
            id: "datasource".to_string(),
            scope: "live".to_string(),
            mode: "live-datasource-inventory".to_string(),
            status: PROJECT_STATUS_READY.to_string(),
            reason_code: PROJECT_STATUS_READY.to_string(),
            primary_count: if org_id == 11 { 1 } else { 2 },
            blocker_count: 0,
            warning_count: 0,
            source_kinds: vec!["live-datasource-list".to_string()],
            signal_keys: vec!["live.datasourceCount".to_string()],
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_actions: Vec::new(),
            freshness: ProjectStatusFreshness {
                status: "current".to_string(),
                source_count: 1,
                newest_age_seconds: Some(10),
                oldest_age_seconds: Some(20),
            },
        }
    })
    .unwrap();

    assert_eq!(seen_org_ids, vec![11, 22]);
    assert_eq!(aggregated.primary_count, 3);
    assert_eq!(aggregated.mode, "live-datasource-inventory-all-orgs");
}

#[test]
fn build_live_multi_org_domain_status_pair_reads_each_scoped_org_once() {
    let clients = vec![
        scoped_live_org_client_for_test(11, test_http_client()),
        scoped_live_org_client_for_test(22, test_http_client()),
    ];
    let mut pass_count = 0usize;

    let (dashboard, datasource) = build_live_multi_org_domain_status_pair(&clients, |_client| {
        pass_count += 1;
        (
            ready_test_domain("dashboard", "live-dashboard-inventory", 1),
            ready_test_domain("datasource", "live-datasource-inventory", 2),
        )
    })
    .unwrap();

    assert_eq!(pass_count, 2);
    assert_eq!(dashboard.primary_count, 2);
    assert_eq!(dashboard.mode, "live-dashboard-inventory-all-orgs");
    assert_eq!(datasource.primary_count, 4);
    assert_eq!(datasource.mode, "live-datasource-inventory-all-orgs");
}

#[test]
fn build_live_dashboard_and_datasource_statuses_reuses_datasource_read() {
    let updated = DateTime::<Utc>::from(SystemTime::now() - Duration::from_secs(60)).to_rfc3339();
    let responses = vec![
        http_response(
            "200 OK",
            &format!(
                r#"[{{"uid":"cpu-main","title":"CPU Main","type":"dash-db","folderUid":"infra","folderTitle":"Infra","updated":"{updated}"}}]"#
            ),
        ),
        http_response(
            "200 OK",
            r#"[{"uid":"prom-main","name":"Prometheus Main","type":"prometheus","orgId":1}]"#,
        ),
        http_response("200 OK", r#"[{"id":1,"name":"Main"}]"#),
        http_response("200 OK", r#"{"id":1,"name":"Main"}"#),
    ];
    let (client, requests, handle) = build_sequence_test_client(responses);

    let (dashboard, datasource) = build_live_dashboard_and_datasource_statuses(&client);
    handle.join().unwrap();
    let requests = requests.lock().unwrap();
    let datasource_reads = requests
        .iter()
        .filter(|line| line.starts_with("GET /api/datasources "))
        .count();

    assert_eq!(dashboard.status, PROJECT_STATUS_READY);
    assert_eq!(datasource.status, PROJECT_STATUS_READY);
    assert_eq!(datasource_reads, 1);
    assert!(requests
        .iter()
        .any(|line| line.starts_with("GET /api/search?")));
    assert!(requests
        .iter()
        .any(|line| line.starts_with("GET /api/orgs ")));
    assert!(requests
        .iter()
        .any(|line| line.starts_with("GET /api/org ")));
}

fn test_http_client() -> JsonHttpClient {
    JsonHttpClient::new(JsonHttpClientConfig {
        base_url: "http://127.0.0.1:3000".to_string(),
        headers: Vec::new(),
        timeout_secs: 1,
        verify_ssl: false,
    })
    .unwrap()
}

fn ready_test_domain(id: &str, mode: &str, primary_count: usize) -> ProjectDomainStatus {
    ProjectDomainStatus {
        id: id.to_string(),
        scope: "live".to_string(),
        mode: mode.to_string(),
        status: PROJECT_STATUS_READY.to_string(),
        reason_code: PROJECT_STATUS_READY.to_string(),
        primary_count,
        blocker_count: 0,
        warning_count: 0,
        source_kinds: vec![format!("live-{id}")],
        signal_keys: vec![format!("live.{id}Count")],
        blockers: Vec::new(),
        warnings: Vec::new(),
        next_actions: Vec::new(),
        freshness: ProjectStatusFreshness {
            status: "current".to_string(),
            source_count: 1,
            newest_age_seconds: Some(10),
            oldest_age_seconds: Some(20),
        },
    }
}

fn build_sequence_test_client(
    responses: Vec<String>,
) -> (
    JsonHttpClient,
    Arc<Mutex<Vec<String>>>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_thread = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
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
            let request_line = String::from_utf8_lossy(&request)
                .lines()
                .next()
                .unwrap_or_default()
                .to_string();
            requests_thread.lock().unwrap().push(request_line);
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    let client = JsonHttpClient::new(JsonHttpClientConfig {
        base_url: format!("http://{address}"),
        headers: Vec::new(),
        timeout_secs: 5,
        verify_ssl: false,
    })
    .unwrap();
    (client, requests, handle)
}

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
