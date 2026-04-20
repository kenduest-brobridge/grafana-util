use super::{
    alert_project_status_freshness_samples, collect_live_dashboard_project_status_inputs,
    collect_live_dashboard_project_status_inputs_with_request,
    dashboard_project_status_freshness_samples, fetch_current_org, fetch_current_org_with_request,
    latest_dashboard_version_timestamp, latest_dashboard_version_timestamp_with_request,
    list_visible_orgs, list_visible_orgs_with_request, load_alert_surface_documents,
    load_alert_surface_documents_with_request, ProjectStatusAlertSurfaceDocuments,
};
use crate::dashboard::DEFAULT_PAGE_SIZE;
use reqwest::Method;
use serde_json::{json, Value};
#[path = "project_status_live_test_support.rs"]
mod test_support;

use test_support::{build_test_client, http_response};

#[test]
fn list_visible_orgs_parses_orgs() {
    let responses = vec![http_response(
        "200 OK",
        r#"[{"id":1,"name":"Main"},{"id":2,"name":"Edge"}]"#,
    )];
    let (client, requests, handle) = build_test_client(responses);
    let orgs = list_visible_orgs(&client).unwrap();
    handle.join().unwrap();

    assert_eq!(orgs.len(), 2);
    assert_eq!(requests.lock().unwrap()[0], "GET /api/orgs HTTP/1.1");
}

#[test]
fn list_visible_orgs_with_request_parses_orgs() {
    let orgs = list_visible_orgs_with_request(&mut |method, path, params, _payload| {
        assert_eq!(method, Method::GET);
        assert_eq!(path, "/api/orgs");
        assert!(params.is_empty());
        Ok(Some(json!([{"id":1,"name":"Main"},{"id":2,"name":"Edge"}])))
    })
    .unwrap();

    assert_eq!(orgs.len(), 2);
}

#[test]
fn fetch_current_org_with_request_parses_org() {
    let org = fetch_current_org_with_request(&mut |method, path, params, _payload| {
        assert_eq!(method, Method::GET);
        assert_eq!(path, "/api/org");
        assert!(params.is_empty());
        Ok(Some(json!({"id":1,"name":"Main"})))
    })
    .unwrap();

    assert_eq!(org.get("name").and_then(Value::as_str), Some("Main"));
}

#[test]
fn fetch_current_org_parses_org() {
    let responses = vec![http_response("200 OK", r#"{"id":1,"name":"Main"}"#)];
    let (client, requests, handle) = build_test_client(responses);
    let org = fetch_current_org(&client).unwrap();
    handle.join().unwrap();

    assert_eq!(org.get("id").and_then(Value::as_i64), Some(1));
    assert_eq!(requests.lock().unwrap()[0], "GET /api/org HTTP/1.1");
}

#[test]
fn latest_dashboard_version_timestamp_uses_first_summary_uid() {
    let responses = vec![http_response(
        "200 OK",
        r#"[{"version":7,"created":"2026-01-01T00:00:00Z"}]"#,
    )];
    let (client, requests, handle) = build_test_client(responses);
    let timestamp = latest_dashboard_version_timestamp(
        &client,
        &[json!({"uid":"cpu-main","title":"CPU"})
            .as_object()
            .unwrap()
            .clone()],
    );
    handle.join().unwrap();

    assert!(timestamp.is_some());
    assert_eq!(
        requests.lock().unwrap()[0],
        "GET /api/dashboards/uid/cpu-main/versions?limit=1 HTTP/1.1"
    );
}

#[test]
fn collect_live_dashboard_project_status_inputs_reads_dashboard_and_datasource_surfaces() {
    let responses = vec![
        http_response(
            "200 OK",
            r#"[{"uid":"cpu-main","title":"CPU Main","folderUid":"infra","folderTitle":"Infra"}]"#,
        ),
        http_response(
            "200 OK",
            r#"[{"uid":"prom-main","name":"Prometheus Main","type":"prometheus"}]"#,
        ),
    ];
    let (client, requests, handle) = build_test_client(responses);
    let inputs = collect_live_dashboard_project_status_inputs(&client, DEFAULT_PAGE_SIZE).unwrap();
    handle.join().unwrap();

    assert_eq!(inputs.dashboard_summaries.len(), 1);
    assert_eq!(inputs.datasources.len(), 1);
    assert_eq!(
        requests.lock().unwrap()[0],
        "GET /api/search?type=dash-db&limit=500&page=1 HTTP/1.1"
    );
    assert_eq!(requests.lock().unwrap()[1], "GET /api/datasources HTTP/1.1");
}

#[test]
fn dashboard_project_status_freshness_samples_collects_dashboard_and_datasource_timestamps() {
    let (client, _, handle) = build_test_client(vec![
        http_response(
            "200 OK",
            r#"[{"uid":"cpu-main","title":"CPU Main","updatedAt":"2026-01-01T00:00:00Z"}]"#,
        ),
        http_response(
            "200 OK",
            r#"[{"uid":"prom-main","name":"Prometheus Main","created":"2026-01-01T01:00:00Z"}]"#,
        ),
    ]);
    let inputs = collect_live_dashboard_project_status_inputs(&client, DEFAULT_PAGE_SIZE).unwrap();
    handle.join().unwrap();

    let samples = dashboard_project_status_freshness_samples(&inputs);

    assert_eq!(samples.len(), 2);
}

#[test]
fn latest_dashboard_version_timestamp_with_request_uses_first_summary_uid() {
    let timestamp = latest_dashboard_version_timestamp_with_request(
        |method, path, params, _payload| {
            assert_eq!(method, Method::GET);
            assert_eq!(path, "/api/dashboards/uid/cpu-main/versions");
            assert_eq!(params, &vec![("limit".to_string(), "1".to_string())]);
            Ok(Some(
                json!([{"version": 7, "created": "2026-01-01T00:00:00Z"}]),
            ))
        },
        &[json!({"uid":"cpu-main","title":"CPU"})
            .as_object()
            .unwrap()
            .clone()],
    );

    assert_eq!(timestamp.as_deref(), Some("2026-01-01T00:00:00Z"));
}

#[test]
fn load_alert_surface_documents_tolerates_null_templates() {
    let responses = vec![
        http_response("200 OK", "[]"),
        http_response("200 OK", "[]"),
        http_response("200 OK", "[]"),
        http_response("200 OK", "{}"),
        http_response("200 OK", "null"),
    ];
    let (client, requests, handle) = build_test_client(responses);
    let docs = load_alert_surface_documents(&client);
    handle.join().unwrap();

    assert!(docs.templates.is_none());
    assert_eq!(requests.lock().unwrap().len(), 5);
}

#[test]
fn load_alert_surface_documents_with_request_tolerates_null_templates() {
    let docs = load_alert_surface_documents_with_request(&mut |method, path, params, _payload| {
        assert_eq!(method, Method::GET);
        assert!(params.is_empty());
        match path {
            "/api/v1/provisioning/alert-rules"
            | "/api/v1/provisioning/contact-points"
            | "/api/v1/provisioning/mute-timings" => Ok(Some(json!([]))),
            "/api/v1/provisioning/policies" => Ok(Some(json!({}))),
            "/api/v1/provisioning/templates" => Ok(Some(Value::Null)),
            _ => Err(crate::common::message(format!("unexpected request {path}"))),
        }
    });

    assert!(docs.templates.is_none());
}

#[test]
fn alert_project_status_freshness_samples_collects_alert_surface_timestamps() {
    let documents = ProjectStatusAlertSurfaceDocuments {
        rules: Some(json!([{"updated":"2026-01-01T00:00:00Z"}])),
        contact_points: Some(json!([{"created":"2026-01-01T01:00:00Z"}])),
        mute_timings: None,
        policies: Some(json!({"modified":"2026-01-01T02:00:00Z"})),
        templates: Some(json!({"createdAt":"2026-01-01T03:00:00Z"})),
    };

    let samples = alert_project_status_freshness_samples(&documents);

    assert_eq!(samples.len(), 4);
}

#[test]
fn collect_live_dashboard_project_status_inputs_with_request_reads_dashboard_and_datasource_surfaces(
) {
    let inputs = collect_live_dashboard_project_status_inputs_with_request(
        &mut |method, path, params, _payload| {
            assert_eq!(method, Method::GET);
            match path {
                "/api/search" => {
                    let page = params
                        .iter()
                        .find(|(key, _)| key == "page")
                        .map(|(_, value)| value.as_str())
                        .unwrap_or("1");
                    if page == "1" {
                        Ok(Some(json!([
                            {"uid":"cpu-main","title":"CPU","folderUid":"infra","folderTitle":"Infra"},
                            {"uid":"cpu-main","title":"CPU","folderUid":"infra","folderTitle":"Infra"},
                            {"uid":"logs-main","title":"Logs","folderUid":"platform","folderTitle":"Platform"}
                        ])))
                    } else {
                        Ok(Some(json!([])))
                    }
                }
                "/api/datasources" => Ok(Some(json!([
                    {"uid":"prom-main","name":"Prometheus"},
                    {"uid":"loki-main","name":"Loki"}
                ]))),
                _ => Err(crate::common::message(format!("unexpected request {path}"))),
            }
        },
        2,
    )
    .unwrap();

    assert_eq!(inputs.dashboard_summaries.len(), 2);
    assert_eq!(inputs.datasources.len(), 2);
    assert_eq!(
        inputs
            .dashboard_summaries
            .first()
            .and_then(|summary| summary.get("uid"))
            .and_then(Value::as_str),
        Some("cpu-main")
    );
    assert_eq!(
        inputs
            .datasources
            .first()
            .and_then(|datasource| datasource.get("uid"))
            .and_then(Value::as_str),
        Some("prom-main")
    );
}
