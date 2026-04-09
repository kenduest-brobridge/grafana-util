//! Focused regression tests for dashboard source-loader contracts.

use super::*;
use std::fs;
use tempfile::tempdir;

fn make_dashboard_repo() -> tempfile::TempDir {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".git")).unwrap();
    fs::create_dir_all(temp.path().join("dashboards")).unwrap();
    temp
}

#[test]
fn source_loader_contract_resolves_direct_raw_root() {
    let temp = make_dashboard_repo();
    let raw_root = temp.path().join("dashboards/raw");
    fs::create_dir_all(&raw_root).unwrap();

    let resolved = load_dashboard_source(&raw_root, DashboardImportInputFormat::Raw, None, true)
        .unwrap();

    assert_eq!(resolved.workspace_root, temp.path());
    assert_eq!(resolved.input_dir, raw_root);
    assert_eq!(resolved.expected_variant, RAW_EXPORT_SUBDIR);
    assert_eq!(resolved.resolved.source_kind, DashboardSourceKind::RawExport);
    assert_eq!(resolved.resolved.dashboard_dir, raw_root);
    assert_eq!(resolved.resolved.metadata_dir, raw_root);
}

#[test]
fn source_loader_contract_resolves_direct_provisioning_root() {
    let temp = make_dashboard_repo();
    let provisioning_root = temp.path().join("dashboards/provisioning");
    fs::create_dir_all(provisioning_root.join("dashboards")).unwrap();

    let resolved = load_dashboard_source(
        &provisioning_root,
        DashboardImportInputFormat::Provisioning,
        None,
        true,
    )
    .unwrap();

    assert_eq!(resolved.workspace_root, temp.path());
    assert_eq!(resolved.input_dir, provisioning_root.join("dashboards"));
    assert_eq!(resolved.expected_variant, "provisioning");
    assert_eq!(
        resolved.resolved.source_kind,
        DashboardSourceKind::ProvisioningExport
    );
    assert_eq!(resolved.resolved.dashboard_dir, provisioning_root.join("dashboards"));
    assert_eq!(resolved.resolved.metadata_dir, provisioning_root);
}

#[test]
fn source_loader_contract_resolves_wrapped_git_sync_repo_root() {
    let temp = make_dashboard_repo();
    let wrapped_raw_root = temp.path().join("dashboards/git-sync/raw");
    fs::create_dir_all(&wrapped_raw_root).unwrap();

    let resolved = load_dashboard_source(&temp.path(), DashboardImportInputFormat::Raw, None, true)
        .unwrap();

    assert_eq!(resolved.workspace_root, temp.path());
    assert_eq!(resolved.input_dir, wrapped_raw_root);
    assert_eq!(resolved.expected_variant, RAW_EXPORT_SUBDIR);
    assert_eq!(resolved.resolved.source_kind, DashboardSourceKind::RawExport);
    assert_eq!(resolved.resolved.dashboard_dir, wrapped_raw_root);
    assert_eq!(resolved.resolved.metadata_dir, wrapped_raw_root);
}

#[test]
fn source_loader_contract_resolves_wrapped_git_sync_tree_from_dashboards_root() {
    let temp = make_dashboard_repo();
    let wrapped_provisioning_root = temp.path().join("dashboards/git-sync/provisioning");
    fs::create_dir_all(&wrapped_provisioning_root).unwrap();

    let dashboards_root = temp.path().join("dashboards");
    assert_eq!(
        resolve_dashboard_workspace_variant_dir(&dashboards_root, "provisioning"),
        Some(wrapped_provisioning_root)
    );
}
