use super::test_support;
use super::TestRequestResult;
use serde_json::Value;

#[allow(clippy::type_complexity)]
pub(crate) fn core_family_inspect_live_request_fixture(
    datasource_inventory: Value,
    dashboard_payload: Value,
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult {
    move |method, path, params, _payload| {
        let method_name = method.to_string();
        match (method, path) {
            (reqwest::Method::GET, "/api/org") => Ok(Some(serde_json::json!({
                "id": 1,
                "name": "Main Org."
            }))),
            (reqwest::Method::GET, "/api/datasources") => Ok(Some(datasource_inventory.clone())),
            (reqwest::Method::GET, "/api/search") => Ok(Some(serde_json::json!([
                {
                    "uid": "core-main",
                    "title": "Core Main",
                    "type": "dash-db",
                    "folderUid": "general",
                    "folderTitle": "General"
                }
            ]))),
            (reqwest::Method::GET, "/api/folders/general") => Ok(Some(serde_json::json!({
                "uid": "general",
                "title": "General"
            }))),
            (reqwest::Method::GET, "/api/folders/general/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/core-main") => {
                Ok(Some(dashboard_payload.clone()))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/core-main/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            _ => Err(test_support::message(format!(
                "unexpected request {method_name} {path} {params:?}"
            ))),
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn all_orgs_inspect_live_request_fixture(
) -> impl FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> TestRequestResult {
    move |method, path, params, _payload| {
        let method_name = method.to_string();
        match (method, path) {
            (reqwest::Method::GET, "/api/orgs") => Ok(Some(serde_json::json!([
                {"id": 1, "name": "Main Org."},
                {"id": 2, "name": "Ops Org"}
            ]))),
            (reqwest::Method::GET, "/api/org") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(serde_json::json!({"id": 1, "name": "Main Org."}))),
                    "2" => Ok(Some(serde_json::json!({"id": 2, "name": "Ops Org"}))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/datasources") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(serde_json::json!([
                        {
                            "uid": "prom-main",
                            "name": "Prometheus Main",
                            "type": "prometheus",
                            "access": "proxy",
                            "url": "http://prometheus:9090",
                            "isDefault": true
                        }
                    ]))),
                    "2" => Ok(Some(serde_json::json!([
                        {
                            "uid": "prom-two",
                            "name": "Prometheus Two",
                            "type": "prometheus",
                            "access": "proxy",
                            "url": "http://prometheus-two:9090",
                            "isDefault": true
                        }
                    ]))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/search") => {
                let scoped_org = params
                    .iter()
                    .find(|(key, _)| key == "orgId")
                    .map(|(_, value)| value.as_str())
                    .unwrap_or("1");
                match scoped_org {
                    "1" => Ok(Some(serde_json::json!([
                        {
                            "uid": "cpu-main",
                            "title": "CPU Main",
                            "type": "dash-db",
                            "folderUid": "general",
                            "folderTitle": "General"
                        }
                    ]))),
                    "2" => Ok(Some(serde_json::json!([
                        {
                            "uid": "latency-main",
                            "title": "Latency Main",
                            "type": "dash-db",
                            "folderUid": "ops",
                            "folderTitle": "Ops"
                        }
                    ]))),
                    other => panic!("unexpected org context {other}"),
                }
            }
            (reqwest::Method::GET, "/api/folders/general") => Ok(Some(
                serde_json::json!({"uid": "general", "title": "General"}),
            )),
            (reqwest::Method::GET, "/api/folders/general/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            (reqwest::Method::GET, "/api/folders/ops") => {
                Ok(Some(serde_json::json!({"uid": "ops", "title": "Ops"})))
            }
            (reqwest::Method::GET, "/api/folders/ops/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main") => Ok(Some(serde_json::json!({
                "dashboard": {
                    "id": 11,
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "panels": [{
                        "id": 7,
                        "title": "CPU Query",
                        "type": "timeseries",
                        "datasource": {"uid": "prom-main", "type": "prometheus"},
                        "targets": [{"refId": "A", "expr": "up"}]
                    }]
                },
                "meta": {"folderUid": "general", "folderTitle": "General"}
            }))),
            (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/latency-main") => {
                Ok(Some(serde_json::json!({
                    "dashboard": {
                        "id": 12,
                        "uid": "latency-main",
                        "title": "Latency Main",
                        "panels": [{
                            "id": 8,
                            "title": "Latency Query",
                            "type": "timeseries",
                            "datasource": {"uid": "prom-two", "type": "prometheus"},
                            "targets": [{"refId": "A", "expr": "rate(http_requests_total[5m])"}]
                        }]
                    },
                    "meta": {"folderUid": "ops", "folderTitle": "Ops"}
                })))
            }
            (reqwest::Method::GET, "/api/dashboards/uid/latency-main/permissions") => {
                Ok(Some(serde_json::json!([])))
            }
            (_method, path) => Err(test_support::message(format!(
                "unexpected request {method_name} {path} {params:?}"
            ))),
        }
    }
}
