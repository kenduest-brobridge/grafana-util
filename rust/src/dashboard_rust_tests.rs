use super::{
    build_export_variant_dirs, build_external_export_document, build_import_payload, build_output_path,
    build_preserved_web_import_document, discover_dashboard_files, export_dashboards_with_request,
    import_dashboards_with_request, CommonCliArgs, ExportArgs, ImportArgs,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn make_common_args(base_url: String) -> CommonCliArgs {
    CommonCliArgs {
        url: base_url,
        api_token: Some("token".to_string()),
        username: None,
        password: None,
        timeout: 30,
        verify_ssl: false,
    }
}

#[test]
fn build_output_path_keeps_folder_structure() {
    let summary = json!({
        "folderTitle": "Infra Team",
        "title": "Cluster Health",
        "uid": "abc",
    });
    let path = build_output_path(Path::new("out"), summary.as_object().unwrap(), false);
    assert_eq!(path, Path::new("out/Infra_Team/Cluster_Health__abc.json"));
}

#[test]
fn build_export_variant_dirs_returns_raw_and_prompt_dirs() {
    let (raw_dir, prompt_dir) = build_export_variant_dirs(Path::new("dashboards"));
    assert_eq!(raw_dir, Path::new("dashboards/raw"));
    assert_eq!(prompt_dir, Path::new("dashboards/prompt"));
}

#[test]
fn discover_dashboard_files_rejects_combined_export_root() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("raw")).unwrap();
    fs::create_dir_all(temp.path().join("prompt")).unwrap();
    let error = discover_dashboard_files(temp.path()).unwrap_err();
    assert!(error.to_string().contains("combined export root"));
}

#[test]
fn build_import_payload_accepts_wrapped_document() {
    let payload = build_import_payload(
        &json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "old-folder"}
        }),
        Some("new-folder"),
        true,
        "sync dashboards",
    )
    .unwrap();

    assert_eq!(payload["dashboard"]["id"], Value::Null);
    assert_eq!(payload["folderUid"], "new-folder");
    assert_eq!(payload["overwrite"], true);
    assert_eq!(payload["message"], "sync dashboards");
}

#[test]
fn build_preserved_web_import_document_clears_numeric_id() {
    let document = build_preserved_web_import_document(&json!({
        "dashboard": {"id": 7, "uid": "abc", "title": "CPU"}
    }))
    .unwrap();

    assert_eq!(document["id"], Value::Null);
    assert_eq!(document["uid"], "abc");
}

#[test]
fn export_dashboards_with_client_writes_raw_variant_and_indexes() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
    };
    let mut calls = Vec::new();
    let count = export_dashboards_with_request(
        |method, path, params, payload| {
            calls.push((method.to_string(), path.to_string(), params.to_vec(), payload.cloned()));
            if path == "/api/search" {
                return Ok(Some(json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }])));
            }
            if path == "/api/dashboards/uid/abc" {
                return Ok(Some(json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU"}})));
            }
            Err(super::message(format!("unexpected path {path}")))
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args.export_dir.join("raw/Infra/CPU__abc.json").is_file());
    assert!(args.export_dir.join("raw/index.json").is_file());
    assert!(args.export_dir.join("index.json").is_file());
    assert_eq!(calls.len(), 2);
}

#[test]
fn build_external_export_document_adds_datasource_inputs() {
    let payload = json!({
        "dashboard": {
            "id": 9,
            "title": "Infra",
            "panels": [
                {
                    "type": "timeseries",
                    "datasource": {"type": "prometheus", "uid": "prom_uid"},
                    "targets": [
                        {
                            "datasource": {"type": "prometheus", "uid": "prom_uid"},
                            "expr": "up"
                        }
                    ]
                },
                {
                    "type": "stat",
                    "datasource": "Loki Logs"
                }
            ]
        }
    });
    let catalog = super::build_datasource_catalog(&vec![
        json!({"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"})
            .as_object()
            .unwrap()
            .clone(),
        json!({"uid": "loki_uid", "name": "Loki Logs", "type": "loki"})
            .as_object()
            .unwrap()
            .clone(),
    ]);

    let document = build_external_export_document(&payload, &catalog).unwrap();

    assert_eq!(document["panels"][0]["datasource"]["uid"], "${DS_PROMETHEUS_1}");
    assert_eq!(document["panels"][0]["targets"][0]["datasource"]["uid"], "${DS_PROMETHEUS_1}");
    assert_eq!(document["panels"][1]["datasource"], "${DS_LOKI_1}");
    assert_eq!(document["__inputs"][0]["name"], "DS_LOKI_1");
    assert_eq!(document["__inputs"][1]["name"], "DS_PROMETHEUS_1");
    assert_eq!(document["__elements"], json!({}));
}

#[test]
fn build_external_export_document_creates_input_from_datasource_template_variable() {
    let payload = json!({
        "dashboard": {
            "id": 15,
            "title": "Prometheus / Overview",
            "templating": {
                "list": [
                    {
                        "current": {"text": "default", "value": "default"},
                        "hide": 0,
                        "label": "Data source",
                        "name": "datasource",
                        "options": [],
                        "query": "prometheus",
                        "refresh": 1,
                        "regex": "",
                        "type": "datasource"
                    },
                    {
                        "allValue": ".+",
                        "current": {"selected": true, "text": "All", "value": "$__all"},
                        "datasource": "$datasource",
                        "includeAll": true,
                        "label": "job",
                        "multi": true,
                        "name": "job",
                        "options": [],
                        "query": "label_values(prometheus_build_info, job)",
                        "refresh": 1,
                        "regex": "",
                        "sort": 2,
                        "type": "query"
                    }
                ]
            },
            "panels": [
                {
                    "type": "timeseries",
                    "datasource": "$datasource",
                    "targets": [{"refId": "A", "expr": "up"}]
                }
            ]
        }
    });

    let document = build_external_export_document(&payload, &(BTreeMap::new(), BTreeMap::new())).unwrap();
    assert_eq!(document["__inputs"][0]["name"], "DS_PROMETHEUS_1");
    assert_eq!(document["templating"]["list"][0]["current"], json!({}));
    assert_eq!(document["templating"]["list"][0]["query"], "prometheus");
    assert_eq!(document["templating"]["list"][1]["datasource"]["uid"], "${DS_PROMETHEUS_1}");
    assert_eq!(document["panels"][0]["datasource"]["uid"], "$datasource");
}

#[test]
fn export_dashboards_with_client_writes_prompt_variant_and_indexes() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        export_dir: temp.path().join("dashboards"),
        page_size: 500,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom_uid", "name": "Prom Main", "type": "prometheus"}
            ]))),
            "/api/search" => Ok(Some(json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]))),
            "/api/dashboards/uid/abc" => Ok(Some(json!({
                "dashboard": {
                    "id": 7,
                    "uid": "abc",
                    "title": "CPU",
                    "panels": [
                        {"type": "timeseries", "datasource": {"type": "prometheus", "uid": "prom_uid"}}
                    ]
                }
            }))),
            _ => Err(super::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args.export_dir.join("prompt/Infra/CPU__abc.json").is_file());
    assert!(args.export_dir.join("prompt/index.json").is_file());
}

#[test]
fn import_dashboards_with_client_imports_discovered_files() {
    let temp = tempdir().unwrap();
    let raw_dir = temp.path().join("raw");
    fs::create_dir_all(&raw_dir).unwrap();
    fs::write(
        raw_dir.join("dash.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {"id": 7, "uid": "abc", "title": "CPU"},
            "meta": {"folderUid": "old-folder"}
        }))
        .unwrap(),
    )
    .unwrap();
    let args = ImportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        import_dir: raw_dir,
        import_folder_uid: Some("new-folder".to_string()),
        replace_existing: true,
        import_message: "sync dashboards".to_string(),
    };
    let mut posted_payloads = Vec::new();
    let count = import_dashboards_with_request(
        |_method, path, _params, payload| {
            assert_eq!(path, "/api/dashboards/db");
            posted_payloads.push(payload.cloned().unwrap());
            Ok(Some(json!({"status": "success"})))
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert_eq!(posted_payloads.len(), 1);
    assert_eq!(posted_payloads[0]["folderUid"], "new-folder");
    assert_eq!(posted_payloads[0]["dashboard"]["id"], Value::Null);
}
