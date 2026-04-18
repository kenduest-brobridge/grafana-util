//! Dashboard domain test suite.
//! Covers parser surfaces, formatter/output contracts, and export/import/inspect/list/diff
//! behavior with in-memory/mocked request fixtures.
#![allow(unused_imports)]

use serde_json::json;
use std::fs;
use tempfile::tempdir;

use super::test_support;
pub(crate) use super::test_support::*;

#[cfg(test)]
#[path = "dashboard_rust_tests_assertions.rs"]
mod dashboard_rust_tests_assertions;
#[cfg(test)]
#[path = "dashboard_rust_tests_fixtures.rs"]
mod dashboard_rust_tests_fixtures;
#[cfg(test)]
#[path = "dashboard_rust_tests_support.rs"]
mod dashboard_rust_tests_support;

#[cfg(test)]
pub(crate) use dashboard_rust_tests_assertions::*;
#[cfg(test)]
pub(crate) use dashboard_rust_tests_fixtures::*;
#[cfg(test)]
pub(crate) use dashboard_rust_tests_support::*;

#[cfg(test)]
#[path = "dashboard_authoring_rust_tests.rs"]
mod dashboard_authoring_rust_tests;
#[cfg(test)]
#[path = "export_diff_rust_tests.rs"]
mod export_diff_rust_tests;
#[cfg(test)]
#[path = "export_diff_tail_rust_tests.rs"]
mod export_diff_tail_rust_tests;
#[cfg(test)]
#[path = "export_focus_report_rust_tests.rs"]
mod export_focus_report_rust_tests;
#[cfg(test)]
#[path = "export_focus_rust_tests.rs"]
mod export_focus_rust_tests;
#[cfg(test)]
#[path = "import_edge_rust_tests.rs"]
mod import_edge_rust_tests;
#[cfg(test)]
#[path = "inspect_live_export_all_orgs_rust_tests.rs"]
mod inspect_live_export_all_orgs_rust_tests;
#[cfg(test)]
#[path = "inspect_live_export_parity_rust_tests.rs"]
mod inspect_live_export_parity_rust_tests;
#[cfg(test)]
#[path = "inspect_live_tui_rust_tests.rs"]
mod inspect_live_tui_rust_tests;
#[cfg(test)]
#[path = "inspect_query_rust_tests.rs"]
mod inspect_query_rust_tests;

#[cfg(test)]
#[path = "dashboard_export_import_topology_import_format_rust_tests.rs"]
mod dashboard_export_import_topology_import_format_rust_tests;

#[test]
fn dashboard_export_root_manifest_classifies_root_scopes() {
    let org_root = DashboardExportRootManifest::from_metadata(build_export_metadata(
        "root",
        1,
        None,
        None,
        None,
        None,
        Some("Main Org."),
        Some("1"),
        None,
        "live",
        Some("http://127.0.0.1:3000"),
        None,
        None,
        std::path::Path::new("/tmp/dashboard-root"),
        std::path::Path::new("/tmp/dashboard-root/export-metadata.json"),
    ));
    assert_eq!(org_root.scope_kind, DashboardExportRootScopeKind::OrgRoot);

    let all_orgs_root = DashboardExportRootManifest::from_metadata(build_export_metadata(
        "root",
        2,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(vec![ExportOrgSummary {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            dashboard_count: 2,
            datasource_count: None,
            used_datasource_count: None,
            used_datasources: None,
            output_dir: None,
        }]),
        "live",
        Some("http://127.0.0.1:3000"),
        None,
        None,
        std::path::Path::new("/tmp/dashboard-root"),
        std::path::Path::new("/tmp/dashboard-root/export-metadata.json"),
    ));
    assert_eq!(
        all_orgs_root.scope_kind,
        DashboardExportRootScopeKind::AllOrgsRoot
    );

    let workspace_root = all_orgs_root
        .clone()
        .with_scope_kind(DashboardExportRootScopeKind::WorkspaceRoot);
    assert_eq!(
        workspace_root.scope_kind,
        DashboardExportRootScopeKind::WorkspaceRoot
    );
    assert!(workspace_root.scope_kind.is_aggregate());
}

#[test]
fn load_dashboard_export_root_manifest_honors_explicit_workspace_scope_kind() {
    let temp = tempdir().unwrap();
    let metadata_path = temp.path().join(EXPORT_METADATA_FILENAME);
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": TOOL_SCHEMA_VERSION,
            "variant": "root",
            "scopeKind": "workspace-root",
            "dashboardCount": 2,
            "indexFile": "index.json",
            "orgCount": 2,
            "orgs": [
                {"org": "Main Org.", "orgId": "1", "dashboardCount": 1},
                {"org": "Ops Org.", "orgId": "2", "dashboardCount": 1}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let manifest = load_dashboard_export_root_manifest(&metadata_path).unwrap();
    assert_eq!(
        manifest.scope_kind,
        DashboardExportRootScopeKind::WorkspaceRoot
    );
}

#[test]
fn resolve_dashboard_export_root_detects_workspace_wrapper_root() {
    let temp = tempdir().unwrap();
    let workspace_root = temp.path().join("workspace");
    let dashboard_root = workspace_root.join("dashboards");
    let metadata_path = dashboard_root.join(EXPORT_METADATA_FILENAME);
    fs::create_dir_all(workspace_root.join("datasources")).unwrap();
    fs::create_dir_all(dashboard_root.join("org_1_Main_Org").join("raw")).unwrap();
    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&build_export_metadata(
            "root",
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(vec![ExportOrgSummary {
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                dashboard_count: 1,
                datasource_count: None,
                used_datasource_count: None,
                used_datasources: None,
                output_dir: None,
            }]),
            "local",
            None,
            Some(std::path::Path::new("/tmp/workspace")),
            None,
            std::path::Path::new("/tmp/workspace/dashboards"),
            &metadata_path,
        ))
        .unwrap(),
    )
    .unwrap();

    let resolved = resolve_dashboard_export_root(&workspace_root)
        .unwrap()
        .expect("workspace root should resolve");
    assert_eq!(resolved.metadata_dir, dashboard_root);
    assert_eq!(
        resolved.manifest.scope_kind,
        DashboardExportRootScopeKind::WorkspaceRoot
    );
}
