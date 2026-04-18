use super::export_layout_render::{
    render_export_layout_plan_csv, render_export_layout_plan_table, render_export_layout_plan_text,
};
use super::*;
use serde_json::json;
use tempfile::tempdir;

fn write_json(path: &Path, value: Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
}

fn write_old_export(root: &Path) {
    write_json(
        &root.join("index.json"),
        json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-dashboard-export-index",
            "items": [
                {
                    "uid": "cpu-main",
                    "title": "CPU",
                    "folderTitle": "Infra",
                    "org": "Main Org.",
                    "orgId": "1",
                    "raw_path": root.join("raw/Infra/CPU__cpu-main.json").display().to_string(),
                    "prompt_path": root.join("prompt/Infra/CPU__cpu-main.json").display().to_string()
                }
            ],
            "variants": {
                "raw": root.join("raw/index.json").display().to_string(),
                "prompt": root.join("prompt/index.json").display().to_string(),
                "provisioning": null
            },
            "folders": []
        }),
    );
    write_json(
        &root.join("raw/export-metadata.json"),
        json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-dashboard-export-index",
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "foldersFile": "folders.json"
        }),
    );
    write_json(
        &root.join("raw/folders.json"),
        json!([
            {
                "uid": "infra",
                "title": "Infra",
                "path": "Platform / Team / Infra",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]),
    );
    write_json(
        &root.join("raw/index.json"),
        json!([
            {
                "uid": "cpu-main",
                "title": "CPU",
                "path": root.join("raw/Infra/CPU__cpu-main.json").display().to_string(),
                "format": "grafana-web-import-preserve-uid",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]),
    );
    write_json(
        &root.join("prompt/export-metadata.json"),
        json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-dashboard-export-index",
            "variant": "prompt",
            "dashboardCount": 1,
            "indexFile": "index.json"
        }),
    );
    write_json(
        &root.join("prompt/index.json"),
        json!([
            {
                "uid": "cpu-main",
                "title": "CPU",
                "path": root.join("prompt/Infra/CPU__cpu-main.json").display().to_string(),
                "format": "grafana-web-import-with-datasource-inputs",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]),
    );
    write_json(
        &root.join("raw/Infra/CPU__cpu-main.json"),
        json!({
            "uid": "cpu-main",
            "title": "CPU",
            "meta": {"folderUid": "infra"}
        }),
    );
    write_json(
        &root.join("prompt/Infra/CPU__cpu-main.json"),
        json!({"uid": "cpu-main", "title": "CPU"}),
    );
}

fn write_old_all_orgs_export(root: &Path) {
    write_json(
        &root.join("export-metadata.json"),
        json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-dashboard-export-index",
            "variant": "root",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "org": "Main Org",
            "orgId": "1"
        }),
    );
    write_json(
        &root.join("index.json"),
        json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-dashboard-export-index",
            "items": [{
                "uid": "cpu-main",
                "title": "CPU",
                "folderTitle": "Infra",
                "folderUid": "infra",
                "folderPath": "Infra",
                "org": "Main Org",
                "orgId": "1",
                "raw_path": root.join("org_1_Main/raw/Infra/CPU__cpu-main.json").display().to_string()
            }],
            "variants": {"raw": null, "prompt": null, "provisioning": null},
            "folders": []
        }),
    );
    for (org_dir, org_name, org_id, uid, title, folder) in [
        ("org_1_Main", "Main Org", "1", "cpu-main", "CPU", "Infra"),
        ("org_2_Ops", "Ops Org", "2", "logs-main", "Logs", "Ops"),
    ] {
        let org_root = root.join(org_dir);
        let raw_dir = org_root.join(RAW_EXPORT_SUBDIR);
        let prompt_dir = org_root.join(PROMPT_EXPORT_SUBDIR);
        write_json(
            &raw_dir.join(EXPORT_METADATA_FILENAME),
            json!({
                "schemaVersion": 1,
                "kind": "grafana-utils-dashboard-export-index",
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "foldersFile": "folders.json",
                "org": org_name,
                "orgId": org_id
            }),
        );
        write_json(
            &raw_dir.join(FOLDER_INVENTORY_FILENAME),
            json!([{
                "uid": folder.to_lowercase(),
                "title": folder,
                "path": folder,
                "org": org_name,
                "orgId": org_id
            }]),
        );
        write_json(
            &raw_dir.join("index.json"),
            json!([{
                "uid": uid,
                "title": title,
                "path": raw_dir.join(folder).join(format!("{title}__{uid}.json")).display().to_string(),
                "format": "grafana-web-import-preserve-uid",
                "folderTitle": folder,
                "folderUid": folder.to_lowercase(),
                "folderPath": folder,
                "org": org_name,
                "orgId": org_id
            }]),
        );
        write_json(
            &prompt_dir.join(EXPORT_METADATA_FILENAME),
            json!({
                "schemaVersion": 1,
                "kind": "grafana-utils-dashboard-export-index",
                "variant": "prompt",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "org": org_name,
                "orgId": org_id
            }),
        );
        write_json(
            &prompt_dir.join("index.json"),
            json!([{
                "uid": uid,
                "title": title,
                "path": prompt_dir.join(folder).join(format!("{title}__{uid}.json")).display().to_string(),
                "format": "grafana-web-import-with-datasource-inputs",
                "folderTitle": folder,
                "folderUid": folder.to_lowercase(),
                "folderPath": folder,
                "org": org_name,
                "orgId": org_id
            }]),
        );
        write_json(
            &raw_dir.join(folder).join(format!("{title}__{uid}.json")),
            json!({"uid": uid, "title": title}),
        );
        write_json(
            &prompt_dir.join(folder).join(format!("{title}__{uid}.json")),
            json!({"uid": uid, "title": title, "__inputs": []}),
        );
    }
}

#[test]
fn export_layout_plan_repairs_raw_and_prompt_from_folder_inventory() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Json,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    assert_eq!(plan.summary.move_count, 2);
    assert_eq!(plan.summary.blocked_count, 0);
    assert!(plan
        .operations
        .iter()
        .any(|operation| operation.to == "Platform/Team/Infra/CPU__cpu-main.json"));
}

#[test]
fn export_layout_plan_uses_unique_root_index_folder_title_when_raw_meta_is_missing() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    write_json(
        &input.join("raw/Infra/CPU__cpu-main.json"),
        json!({
            "uid": "cpu-main",
            "title": "CPU"
        }),
    );
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Json,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    assert_eq!(plan.summary.move_count, 2);
    assert_eq!(plan.summary.blocked_count, 0);
    assert!(plan
        .operations
        .iter()
        .all(|operation| operation.folder_uid.as_deref() == Some("infra")));
}

#[test]
fn export_layout_plan_reports_files_not_listed_in_variant_index() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    write_json(
        &input.join("raw/Infra/Manual__manual.json"),
        json!({"uid": "manual", "title": "Manual"}),
    );
    fs::write(input.join("prompt/Infra/operator-note.txt"), "local note\n").unwrap();
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Json,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    assert_eq!(plan.summary.extra_file_count, 2);
    assert!(plan
        .extra_files
        .iter()
        .any(|file| file.path == "Infra/Manual__manual.json"));
    assert!(plan
        .extra_files
        .iter()
        .any(|file| file.path == "Infra/operator-note.txt"));
}

#[test]
fn export_layout_entry_path_keeps_folder_segments_named_like_variant() {
    let variant_dir = Path::new("/tmp/export/raw");
    let relative = entry_relative_path(
        "dashboards/org_1_Main_Org/raw/Team/raw/CPU__cpu-main.json",
        variant_dir,
        LayoutVariant::Raw,
    )
    .unwrap();

    assert_eq!(relative, PathBuf::from("Team/raw/CPU__cpu-main.json"));
}

#[test]
fn export_layout_text_output_defaults_to_summary_only() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Text,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };
    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    let output = render_export_layout_plan_text(&plan, false);

    assert!(output.contains("Dashboards: 2"));
    assert!(output.contains("Operations: 2 move, 0 same, 0 blocked, 0 extra"));
    assert!(!output.contains("MOVE raw uid=cpu-main"));
}

#[test]
fn export_layout_text_output_show_operations_includes_operations() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: true,
        output_format: ExportLayoutOutputFormat::Text,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };
    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    let output = render_export_layout_plan_text(&plan, true);

    assert!(output.contains("Dashboards: 2"));
    assert!(output.contains("Operations\n"));
    assert!(output.contains("MOVE raw uid=cpu-main"));
    assert!(output.contains("Infra/CPU__cpu-main.json -> Platform/Team/Infra/CPU__cpu-main.json"));
}

#[test]
fn export_layout_table_output_defaults_to_summary_rows() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Table,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };
    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    let output = render_export_layout_plan_table(&plan, false, true).join("\n");

    assert!(output.contains("FIELD"));
    assert!(output.contains("dashboards"));
    assert!(output.contains("move"));
    assert!(!output.contains("ACTION"));
    assert!(!output.contains("cpu-main"));
}

#[test]
fn export_layout_csv_output_supports_summary_and_operation_rows() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(temp.path().join("fixed")),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: true,
        overwrite: false,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Csv,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };
    let plan = build_export_layout_plan(&args, &selected_variants(&args)).unwrap();

    let summary = render_export_layout_plan_csv(&plan, false);
    let operations = render_export_layout_plan_csv(&plan, true);

    assert!(summary.starts_with("dashboards,variants,move,same,blocked,extra,output\n"));
    assert!(summary.contains("2,raw|prompt,2,0,0,0,"));
    assert!(operations.starts_with("action,variant,uid,from,to,reason\n"));
    assert!(operations.contains("move,raw,cpu-main"));
}

#[test]
fn export_layout_repair_copy_mode_updates_files_and_indexes() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    let output = temp.path().join("fixed");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input,
        output_dir: Some(output.clone()),
        in_place: false,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: false,
        overwrite: true,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Text,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    run_export_layout_repair(&args).unwrap();

    assert!(output
        .join("raw/Platform/Team/Infra/CPU__cpu-main.json")
        .is_file());
    assert!(output
        .join("prompt/Platform/Team/Infra/CPU__cpu-main.json")
        .is_file());
    let raw_index: Vec<VariantIndexEntry> =
        serde_json::from_str(&fs::read_to_string(output.join("raw/index.json")).unwrap()).unwrap();
    assert!(raw_index[0].path.contains("Platform/Team/Infra"));
    let root_index: RootExportIndex =
        serde_json::from_str(&fs::read_to_string(output.join("index.json")).unwrap()).unwrap();
    assert!(root_index.items[0]
        .raw_path
        .as_ref()
        .unwrap()
        .contains("Platform/Team/Infra"));
}

#[test]
fn export_layout_repair_rebuilds_legacy_all_orgs_root_aggregate() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    write_old_all_orgs_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input.clone(),
        output_dir: None,
        in_place: true,
        backup_dir: None,
        variant: Vec::new(),
        raw_dir: None,
        folders_file: None,
        dry_run: false,
        overwrite: true,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Text,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    run_export_layout_repair(&args).unwrap();

    let root_index: RootExportIndex =
        serde_json::from_str(&fs::read_to_string(input.join("index.json")).unwrap()).unwrap();
    assert_eq!(root_index.items.len(), 2);
    assert!(root_index.items.iter().any(|item| item.org_id == "1"));
    assert!(root_index.items.iter().any(|item| item.org_id == "2"));
    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(input.join(EXPORT_METADATA_FILENAME)).unwrap())
            .unwrap();
    assert_eq!(metadata["scopeKind"], "all-orgs-root");
    assert_eq!(metadata["source"]["orgScope"], "all-orgs");
    assert_eq!(metadata["dashboardCount"], 2);
    assert_eq!(metadata["orgCount"], 2);
    assert!(metadata.get("org").is_none());
    assert!(metadata.get("orgId").is_none());
    assert!(metadata["orgs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["orgId"] == "2" && entry["dashboardCount"] == 1));
}

#[test]
fn export_layout_repair_in_place_writes_backup_before_move() {
    let temp = tempdir().unwrap();
    let input = temp.path().join("dashboards");
    let backup = temp.path().join("backup");
    write_old_export(&input);
    let args = ExportLayoutArgs {
        input_dir: input.clone(),
        output_dir: None,
        in_place: true,
        backup_dir: Some(backup.clone()),
        variant: vec![ExportLayoutVariant::Raw],
        raw_dir: None,
        folders_file: None,
        dry_run: false,
        overwrite: true,
        show_operations: false,
        output_format: ExportLayoutOutputFormat::Text,
        no_header: false,
        color: crate::common::CliColorChoice::Never,
    };

    run_export_layout_repair(&args).unwrap();

    assert!(input
        .join("raw/Platform/Team/Infra/CPU__cpu-main.json")
        .is_file());
    assert!(backup.join("raw/Infra/CPU__cpu-main.json").is_file());
    assert!(input.join("prompt/Infra/CPU__cpu-main.json").is_file());
}
