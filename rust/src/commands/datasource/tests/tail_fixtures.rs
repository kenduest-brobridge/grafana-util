//! Shared datasource tail fixture builders.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn write_diff_fixture(records: &[Value]) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let sequence = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "grafana-util-datasource-diff-{}-{unique}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("export-metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "root",
            "resource": "datasource",
            "datasourcesFile": "datasources.json",
            "indexFile": "index.json",
            "datasourceCount": records.len(),
            "format": "grafana-datasource-masked-recovery-v1",
            "exportMode": "masked-recovery",
            "masked": true,
            "recoveryCapable": true,
            "secretMaterial": "placeholders-only",
            "provisioningProjection": "derived-projection"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("datasources.json"),
        serde_json::to_vec_pretty(&Value::Array(records.to_vec())).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("index.json"),
        serde_json::to_vec_pretty(&json!({"items": []})).unwrap(),
    )
    .unwrap();
    dir
}

pub(super) fn write_provisioning_diff_fixture() -> std::path::PathBuf {
    let root = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let provisioning_dir = root.join("provisioning");
    fs::create_dir_all(&provisioning_dir).unwrap();
    fs::write(
        provisioning_dir.join("datasources.yaml"),
        r#"apiVersion: 1
datasources:
  - uid: prom-main
    name: Prometheus Main
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    jsonData:
      httpMethod: POST
    isDefault: true
    orgId: 1
"#,
    )
    .unwrap();
    root
}

pub(super) fn write_multi_org_import_fixture(
    root: &Path,
    orgs: &[(i64, &str, Vec<Value>)],
) -> std::path::PathBuf {
    let import_root = root.join("datasource-export-all-orgs");
    fs::create_dir_all(&import_root).unwrap();
    let total_datasource_count = orgs
        .iter()
        .map(|(_, _, records)| records.len())
        .sum::<usize>();
    fs::write(
        import_root.join("export-metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "all-orgs-root",
            "scopeKind": "all-orgs-root",
            "resource": "datasource",
            "indexFile": "index.json",
            "datasourceCount": total_datasource_count,
            "orgCount": orgs.len(),
            "format": "grafana-datasource-masked-recovery-v1",
            "exportMode": "masked-recovery",
            "masked": true,
            "recoveryCapable": true,
            "secretMaterial": "placeholders-only",
            "provisioningProjection": "derived-projection"
        }))
        .unwrap(),
    )
    .unwrap();
    for (org_id, org_name, records) in orgs {
        let org_dir = import_root.join(format!("org_{}_{}", org_id, org_name.replace(' ', "_")));
        fs::create_dir_all(&org_dir).unwrap();
        fs::write(
            org_dir.join("export-metadata.json"),
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": 1,
                "kind": "grafana-utils-datasource-export-index",
                "variant": "root",
                "scopeKind": "org-root",
                "resource": "datasource",
                "datasourcesFile": "datasources.json",
                "indexFile": "index.json",
                "datasourceCount": records.len(),
                "format": "grafana-datasource-masked-recovery-v1",
                "exportMode": "masked-recovery",
                "masked": true,
                "recoveryCapable": true,
                "secretMaterial": "placeholders-only",
                "provisioningProjection": "derived-projection"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            org_dir.join("datasources.json"),
            serde_json::to_vec_pretty(&Value::Array(records.clone())).unwrap(),
        )
        .unwrap();
        let index_items = records
            .iter()
            .map(|record| {
                let object = record.as_object().unwrap();
                json!({
                    "uid": object.get("uid").cloned().unwrap_or(Value::String(String::new())),
                    "name": object.get("name").cloned().unwrap_or(Value::String(String::new())),
                    "type": object.get("type").cloned().unwrap_or(Value::String(String::new())),
                    "org": object.get("org").cloned().unwrap_or(Value::String(org_name.to_string())),
                    "orgId": object.get("orgId").cloned().unwrap_or(Value::String(org_id.to_string())),
                })
            })
            .collect::<Vec<Value>>();
        fs::write(
            org_dir.join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "kind": "grafana-utils-datasource-export-index",
                "schemaVersion": 1,
                "datasourcesFile": "datasources.json",
                "exportMode": "masked-recovery",
                "masked": true,
                "recoveryCapable": true,
                "secretMaterial": "placeholders-only",
                "variants": {
                    "inventory": "datasources.json",
                    "provisioning": "provisioning/datasources.yaml"
                },
                "count": records.len(),
                "items": index_items
            }))
            .unwrap(),
        )
        .unwrap();
    }
    import_root
}
