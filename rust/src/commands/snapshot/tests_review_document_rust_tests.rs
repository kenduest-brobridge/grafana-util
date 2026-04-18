use super::super::tests_fixtures::{
    write_complete_dashboard_scope, write_datasource_inventory_rows,
    write_datasource_provisioning_lane, write_snapshot_access_lane_bundle,
    write_snapshot_dashboard_index, write_snapshot_dashboard_metadata,
    write_snapshot_datasource_root_metadata,
};
use crate::snapshot::build_snapshot_review_document;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn snapshot_review_document_summarizes_inventory_counts_without_actions() {
    let temp = tempdir().unwrap();
    let snapshot_root = temp.path().join("snapshot");
    let dashboard_root = snapshot_root.join("dashboards");
    let datasource_root = snapshot_root.join("datasources");
    let access_root = snapshot_root.join("access");

    write_snapshot_dashboard_metadata(
        &dashboard_root,
        &[("1", "Main Org.", 2), ("2", "Ops Org", 1)],
    );
    write_complete_dashboard_scope(&dashboard_root.join("org_1_Main_Org"));
    write_complete_dashboard_scope(&dashboard_root.join("org_2_Ops_Org"));
    write_snapshot_dashboard_index(
        &dashboard_root,
        &[json!({
            "title": "Platform",
            "path": "Platform / Infra",
            "uid": "platform",
            "org": "Main Org.",
            "orgId": "1"
        })],
    );
    write_snapshot_datasource_root_metadata(&datasource_root, 3, "root");
    write_datasource_inventory_rows(
        &datasource_root,
        &[
            json!({
                "uid": "prom-main",
                "name": "prom-main",
                "type": "prometheus",
                "url": "http://prometheus:9090",
                "isDefault": true,
                "org": "Main Org.",
                "orgId": "1"
            }),
            json!({
                "uid": "loki-main",
                "name": "loki-main",
                "type": "loki",
                "url": "http://loki:3100",
                "isDefault": false,
                "org": "Main Org.",
                "orgId": "1"
            }),
            json!({
                "uid": "tempo-ops",
                "name": "tempo-ops",
                "type": "tempo",
                "url": "http://tempo:3200",
                "isDefault": false,
                "org": "Ops Org",
                "orgId": "2"
            }),
        ],
    );
    write_datasource_provisioning_lane(&datasource_root);
    write_snapshot_access_lane_bundle(
        &access_root.join("users"),
        "users.json",
        "grafana-utils-access-user-export-index",
        2,
    );
    write_snapshot_access_lane_bundle(
        &access_root.join("teams"),
        "teams.json",
        "grafana-utils-access-team-export-index",
        3,
    );
    write_snapshot_access_lane_bundle(
        &access_root.join("orgs"),
        "orgs.json",
        "grafana-utils-access-org-export-index",
        1,
    );
    write_snapshot_access_lane_bundle(
        &access_root.join("service-accounts"),
        "service-accounts.json",
        "grafana-utils-access-service-account-export-index",
        4,
    );

    let document =
        build_snapshot_review_document(&dashboard_root, &datasource_root, &datasource_root)
            .unwrap();
    assert_eq!(document["kind"], json!("grafana-utils-snapshot-review"));
    assert_eq!(document["summary"]["orgCount"], json!(2));
    assert_eq!(document["summary"]["dashboardOrgCount"], json!(2));
    assert_eq!(document["summary"]["datasourceOrgCount"], json!(2));
    assert_eq!(document["summary"]["dashboardCount"], json!(3));
    assert_eq!(document["summary"]["datasourceCount"], json!(3));
    assert_eq!(document["summary"]["folderCount"], json!(1));
    assert_eq!(document["summary"]["datasourceTypeCount"], json!(3));
    assert_eq!(document["summary"]["accessUserCount"], json!(2));
    assert_eq!(document["summary"]["accessTeamCount"], json!(3));
    assert_eq!(document["summary"]["accessOrgCount"], json!(1));
    assert_eq!(document["summary"]["accessServiceAccountCount"], json!(4));
    let warning_codes: Vec<&str> = document["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|warning| warning["code"].as_str().unwrap())
        .collect();
    assert!(
        warning_codes.is_empty(),
        "unexpected warnings: {warning_codes:?}"
    );

    let orgs = document["orgs"].as_array().expect("orgs");
    assert_eq!(orgs.len(), 2);
    assert_eq!(orgs[0]["org"], json!("Main Org."));
    assert_eq!(orgs[0]["dashboardCount"], json!(2));
    assert_eq!(orgs[0]["datasourceCount"], json!(2));
    assert_eq!(orgs[1]["org"], json!("Ops Org"));
    assert_eq!(orgs[1]["dashboardCount"], json!(1));
    assert_eq!(orgs[1]["datasourceCount"], json!(1));
}
