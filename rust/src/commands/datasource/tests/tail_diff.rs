//! Datasource tail diff parser and live comparison tests.

use super::*;

#[test]
fn parse_datasource_diff_preserves_requested_path() {
    let args =
        DatasourceCliArgs::parse_from(["grafana-util", "diff", "--diff-dir", "./datasources"]);

    match args.command {
        DatasourceGroupCommand::Diff(inner) => {
            assert_eq!(inner.diff_dir, Path::new("./datasources"));
            assert_eq!(inner.input_format, DatasourceImportInputFormat::Inventory);
        }
        _ => panic!("expected datasource diff"),
    }
}

#[test]
fn parse_datasource_diff_supports_provisioning_input_format() {
    let args = DatasourceCliArgs::parse_from([
        "grafana-util",
        "diff",
        "--diff-dir",
        "./datasources/provisioning",
        "--input-format",
        "provisioning",
    ]);

    match args.command {
        DatasourceGroupCommand::Diff(inner) => {
            assert_eq!(inner.diff_dir, Path::new("./datasources/provisioning"));
            assert_eq!(
                inner.input_format,
                DatasourceImportInputFormat::Provisioning
            );
        }
        _ => panic!("expected datasource diff"),
    }
}

#[test]
fn diff_datasources_with_live_returns_zero_for_matching_inventory() {
    let diff_dir = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let (compared_count, differences) = diff_datasources_with_live(
        &diff_dir,
        DatasourceImportInputFormat::Inventory,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();

    assert_eq!(compared_count, 1);
    assert_eq!(differences, 0);
    fs::remove_dir_all(diff_dir).unwrap();
}

#[test]
fn diff_datasources_with_live_detects_changed_inventory() {
    let diff_dir = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "direct",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let (compared_count, differences) = diff_datasources_with_live(
        &diff_dir,
        DatasourceImportInputFormat::Inventory,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();

    assert_eq!(compared_count, 1);
    assert_eq!(differences, 1);
    fs::remove_dir_all(diff_dir).unwrap();
}

#[test]
fn diff_datasources_with_live_supports_provisioning_root_directory_and_file() {
    let diff_root = write_provisioning_diff_fixture();
    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let provisioning_dir = diff_root.join("provisioning");
    let provisioning_file = provisioning_dir.join("datasources.yaml");

    let (root_count, root_differences) = diff_datasources_with_live(
        &diff_root,
        DatasourceImportInputFormat::Provisioning,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();
    let (dir_count, dir_differences) = diff_datasources_with_live(
        &provisioning_dir,
        DatasourceImportInputFormat::Provisioning,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();
    let (file_count, file_differences) = diff_datasources_with_live(
        &provisioning_file,
        DatasourceImportInputFormat::Provisioning,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();

    assert_eq!((root_count, root_differences), (1, 0));
    assert_eq!((dir_count, dir_differences), (1, 0));
    assert_eq!((file_count, file_differences), (1, 0));
    fs::remove_dir_all(diff_root).unwrap();
}

#[test]
fn diff_datasources_with_live_supports_workspace_root_inventory_directory() {
    let temp = tempdir().unwrap();
    let workspace_root = temp.path().join("workspace");
    let datasource_export_root = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    fs::create_dir_all(&workspace_root).unwrap();
    let datasource_root = workspace_root.join("datasources");
    fs::rename(&datasource_export_root, &datasource_root).unwrap();
    fs::create_dir_all(workspace_root.join("dashboards")).unwrap();

    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let (compared_count, differences) = diff_datasources_with_live(
        &workspace_root,
        DatasourceImportInputFormat::Inventory,
        &live,
        crate::common::DiffOutputFormat::Text,
    )
    .unwrap();

    assert_eq!((compared_count, differences), (1, 0));
}
