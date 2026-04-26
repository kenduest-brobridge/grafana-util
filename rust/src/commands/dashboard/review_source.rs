//! Shared source resolution for dashboard review commands.

use serde_json::Value;
use std::path::Path;

use crate::common::{load_json_object_file, message, Result};

use super::cli_defs::{CommonCliArgs, DashboardImportInputFormat, InspectExportInputType};
use super::export::export_dashboards_with_org_clients;
use super::inspect::{
    build_export_inspection_query_report_for_variant, build_export_inspection_summary_for_variant,
};
use super::inspect_governance::build_export_inspection_governance_document;
use super::inspect_live::{
    build_review_live_export_args, prepare_live_review_import_dir, TempInspectDir,
};
use super::inspect_report::build_export_inspection_query_report_document;
use super::source_loader::load_dashboard_source;
use super::ExportArgs;

#[derive(Debug)]
pub(crate) struct DashboardReviewArtifacts {
    pub(crate) governance: Value,
    pub(crate) queries: Value,
}

#[derive(Debug, Clone)]
pub(crate) enum DashboardReviewSource<'a> {
    ExportTree(DashboardExportTreeSource<'a>),
    SavedArtifacts(DashboardSavedArtifactsSource<'a>),
    Live(DashboardLiveReviewSource<'a>),
}

#[derive(Debug, Clone)]
pub(crate) struct DashboardExportTreeSource<'a> {
    pub(crate) input_dir: &'a Path,
    pub(crate) input_format: DashboardImportInputFormat,
    pub(crate) input_type: Option<InspectExportInputType>,
}

#[derive(Debug, Clone)]
pub(crate) struct DashboardSavedArtifactsSource<'a> {
    pub(crate) governance: &'a Path,
    pub(crate) queries: Option<&'a Path>,
}

#[derive(Debug, Clone)]
pub(crate) struct DashboardLiveReviewSource<'a> {
    pub(crate) common: &'a CommonCliArgs,
    pub(crate) page_size: usize,
    pub(crate) org_id: Option<i64>,
    pub(crate) all_orgs: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DashboardReviewSourceArgs<'a> {
    pub(crate) common: &'a CommonCliArgs,
    pub(crate) page_size: usize,
    pub(crate) org_id: Option<i64>,
    pub(crate) all_orgs: bool,
    pub(crate) input_dir: Option<&'a Path>,
    pub(crate) input_format: DashboardImportInputFormat,
    pub(crate) input_type: Option<InspectExportInputType>,
    pub(crate) governance: Option<&'a Path>,
    pub(crate) queries: Option<&'a Path>,
    pub(crate) require_queries: bool,
}

fn load_object(path: &Path, label: &str) -> Result<Value> {
    load_json_object_file(path, label)
}

fn build_artifacts_from_export_dir(
    input_dir: &Path,
    input_format: DashboardImportInputFormat,
    input_type: Option<InspectExportInputType>,
) -> Result<DashboardReviewArtifacts> {
    let resolved = load_dashboard_source(input_dir, input_format, input_type, false)?;
    let summary = build_export_inspection_summary_for_variant(
        &resolved.input_dir,
        resolved.expected_variant,
    )?;
    let report = build_export_inspection_query_report_for_variant(
        &resolved.input_dir,
        resolved.expected_variant,
    )?;
    Ok(DashboardReviewArtifacts {
        governance: serde_json::to_value(build_export_inspection_governance_document(
            &summary, &report,
        ))?,
        queries: serde_json::to_value(build_export_inspection_query_report_document(&report))?,
    })
}

fn build_artifacts_from_live(
    source: &DashboardLiveReviewSource<'_>,
) -> Result<DashboardReviewArtifacts> {
    let temp_dir = TempInspectDir::new("dashboard-review-live")?;
    let export_args: ExportArgs = build_review_live_export_args(
        source.common,
        temp_dir.path.clone(),
        source.page_size,
        source.org_id,
        source.all_orgs,
    );
    let _ = export_dashboards_with_org_clients(&export_args)?;
    let input_dir = prepare_live_review_import_dir(&temp_dir.path, source.all_orgs)?;
    build_artifacts_from_export_dir(
        &input_dir,
        DashboardImportInputFormat::Raw,
        Some(InspectExportInputType::Raw),
    )
}

fn build_artifacts_from_saved(
    source: &DashboardSavedArtifactsSource<'_>,
) -> Result<DashboardReviewArtifacts> {
    let governance = load_object(source.governance, "Dashboard governance JSON")?;
    let queries = match source.queries {
        Some(path) => load_object(path, "Dashboard query report JSON")?,
        None => Value::Null,
    };
    Ok(DashboardReviewArtifacts {
        governance,
        queries,
    })
}

pub(crate) fn resolve_dashboard_review_source<'a>(
    source: &DashboardReviewSourceArgs<'a>,
) -> Result<DashboardReviewSource<'a>> {
    if let Some(input_dir) = source.input_dir {
        if source.governance.is_some() || source.queries.is_some() {
            return Err(message(
                "--input-dir cannot be combined with --governance or --queries.",
            ));
        }
        return Ok(DashboardReviewSource::ExportTree(
            DashboardExportTreeSource {
                input_dir,
                input_format: source.input_format,
                input_type: source.input_type,
            },
        ));
    }

    if source.governance.is_some() || source.queries.is_some() {
        let governance_path = source.governance.ok_or_else(|| {
            message("--governance is required when reusing saved dashboard review artifacts.")
        })?;
        if source.require_queries && source.queries.is_none() {
            return Err(message(
                "--queries is required when reusing saved dashboard review artifacts for policy.",
            ));
        }
        return Ok(DashboardReviewSource::SavedArtifacts(
            DashboardSavedArtifactsSource {
                governance: governance_path,
                queries: source.queries,
            },
        ));
    }

    Ok(DashboardReviewSource::Live(DashboardLiveReviewSource {
        common: source.common,
        page_size: source.page_size,
        org_id: source.org_id,
        all_orgs: source.all_orgs,
    }))
}

pub(crate) fn resolve_dashboard_review_artifacts(
    source: &DashboardReviewSourceArgs<'_>,
) -> Result<DashboardReviewArtifacts> {
    match resolve_dashboard_review_source(source)? {
        DashboardReviewSource::ExportTree(export_tree) => build_artifacts_from_export_dir(
            export_tree.input_dir,
            export_tree.input_format,
            export_tree.input_type,
        ),
        DashboardReviewSource::SavedArtifacts(saved) => build_artifacts_from_saved(&saved),
        DashboardReviewSource::Live(live) => build_artifacts_from_live(&live),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::CliColorChoice;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn make_common_args() -> CommonCliArgs {
        CommonCliArgs {
            color: CliColorChoice::Never,
            profile: None,
            url: "http://127.0.0.1:3000".to_string(),
            api_token: None,
            username: None,
            password: None,
            prompt_password: false,
            prompt_token: false,
            timeout: 30,
            verify_ssl: false,
        }
    }

    fn write_basic_raw_export(
        raw_dir: &Path,
        dashboard_uid: &str,
        dashboard_title: &str,
        datasource_uid: &str,
    ) {
        fs::create_dir_all(raw_dir).unwrap();
        fs::write(
            raw_dir.join("export-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-export-index",
                "schemaVersion": 1,
                "variant": "raw",
                "dashboardCount": 1,
                "indexFile": "index.json",
                "format": "grafana-web-import-preserve-uid",
                "foldersFile": "folders.json",
                "datasourcesFile": "datasources.json",
                "org": "Main Org",
                "orgId": "1"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("folders.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": "general",
                    "title": "General",
                    "path": "General",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("datasources.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": datasource_uid,
                    "name": "Prometheus Main",
                    "type": "prometheus",
                    "access": "proxy",
                    "url": "http://grafana.example.internal",
                    "isDefault": "true",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("index.json"),
            serde_json::to_string_pretty(&json!([
                {
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "path": "dash.json",
                    "format": "grafana-web-import-preserve-uid",
                    "org": "Main Org",
                    "orgId": "1"
                }
            ]))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            raw_dir.join("dash.json"),
            serde_json::to_string_pretty(&json!({
                "dashboard": {
                    "id": null,
                    "uid": dashboard_uid,
                    "title": dashboard_title,
                    "schemaVersion": 38,
                    "templating": {
                        "list": [
                            { "name": "env", "type": "query", "datasource": { "uid": datasource_uid, "type": "prometheus" } }
                        ]
                    },
                    "panels": [{
                        "id": 7,
                        "title": "CPU",
                        "type": "timeseries",
                        "datasource": {"uid": datasource_uid, "type": "prometheus"},
                        "targets": [{
                            "refId": "A",
                            "expr": "sum(rate(cpu_seconds_total[5m]))"
                        }]
                    }]
                },
                "meta": {
                    "folderUid": "general",
                    "folderTitle": "General"
                }
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn resolve_dashboard_review_artifacts_from_import_dir_builds_documents() {
        let temp = tempdir().unwrap();
        let raw_dir = temp.path().join("raw");
        write_basic_raw_export(&raw_dir, "cpu-main", "CPU Main", "prom-main");
        let common = make_common_args();

        let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: Some(&raw_dir),
            input_format: DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: None,
            queries: None,
            require_queries: false,
        })
        .unwrap();

        assert_eq!(artifacts.governance["summary"]["dashboardCount"], json!(1));
        assert_eq!(artifacts.queries["summary"]["dashboardCount"], json!(1));
        assert_eq!(
            artifacts.queries["queries"][0]["dashboardUid"],
            json!("cpu-main")
        );
    }

    #[test]
    fn resolve_dashboard_review_source_classifies_export_tree_inputs() {
        let temp = tempdir().unwrap();
        let raw_dir = temp.path().join("raw");
        let common = make_common_args();

        let source = resolve_dashboard_review_source(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: Some(&raw_dir),
            input_format: DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: None,
            queries: None,
            require_queries: false,
        })
        .unwrap();

        match source {
            DashboardReviewSource::ExportTree(export_tree) => {
                assert_eq!(export_tree.input_dir, raw_dir.as_path());
                assert_eq!(export_tree.input_format, DashboardImportInputFormat::Raw);
                assert_eq!(export_tree.input_type, Some(InspectExportInputType::Raw));
            }
            _ => panic!("expected export tree source"),
        }
    }

    #[test]
    fn resolve_dashboard_review_source_rejects_mixed_export_and_saved_artifact_inputs() {
        let temp = tempdir().unwrap();
        let raw_dir = temp.path().join("raw");
        let governance_path = temp.path().join("governance.json");
        let common = make_common_args();

        let error = resolve_dashboard_review_source(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: Some(&raw_dir),
            input_format: DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: Some(&governance_path),
            queries: None,
            require_queries: false,
        })
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("--input-dir cannot be combined with --governance or --queries"));
    }

    #[test]
    fn resolve_dashboard_review_source_classifies_saved_artifacts_without_query_report() {
        let temp = tempdir().unwrap();
        let governance_path = temp.path().join("governance.json");
        let common = make_common_args();

        let source = resolve_dashboard_review_source(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: None,
            input_format: DashboardImportInputFormat::Raw,
            input_type: None,
            governance: Some(&governance_path),
            queries: None,
            require_queries: false,
        })
        .unwrap();

        match source {
            DashboardReviewSource::SavedArtifacts(saved) => {
                assert_eq!(saved.governance, governance_path.as_path());
                assert!(saved.queries.is_none());
            }
            _ => panic!("expected saved artifact source"),
        }
    }

    #[test]
    fn resolve_dashboard_review_artifacts_reuses_saved_review_artifacts_with_query_report() {
        let temp = tempdir().unwrap();
        let governance_path = temp.path().join("governance.json");
        let queries_path = temp.path().join("queries.json");
        fs::write(
            &governance_path,
            serde_json::to_string_pretty(&json!({
                "summary": {"dashboardCount": 1},
                "dashboardGovernance": [],
                "dashboardDatasourceEdges": []
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &queries_path,
            serde_json::to_string_pretty(&json!({
                "summary": {"dashboardCount": 1, "queryRecordCount": 0},
                "queries": []
            }))
            .unwrap(),
        )
        .unwrap();
        let common = make_common_args();

        let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: None,
            input_format: DashboardImportInputFormat::Raw,
            input_type: None,
            governance: Some(&governance_path),
            queries: Some(&queries_path),
            require_queries: true,
        })
        .unwrap();

        assert_eq!(artifacts.governance["summary"]["dashboardCount"], json!(1));
        assert_eq!(artifacts.queries["summary"]["queryRecordCount"], json!(0));
        assert_eq!(artifacts.queries["queries"], json!([]));
    }

    #[test]
    fn resolve_dashboard_review_source_classifies_live_inputs_when_no_files_are_given() {
        let common = make_common_args();

        let source = resolve_dashboard_review_source(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 250,
            org_id: Some(7),
            all_orgs: true,
            input_dir: None,
            input_format: DashboardImportInputFormat::Raw,
            input_type: None,
            governance: None,
            queries: None,
            require_queries: false,
        })
        .unwrap();

        match source {
            DashboardReviewSource::Live(live) => {
                assert_eq!(live.page_size, 250);
                assert_eq!(live.org_id, Some(7));
                assert!(live.all_orgs);
                assert_eq!(live.common.url, "http://127.0.0.1:3000");
            }
            _ => panic!("expected live source"),
        }
    }

    #[test]
    fn resolve_dashboard_review_artifacts_supports_git_sync_repo_layout() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path();
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let raw_dir = repo_root.join("dashboards/git-sync/raw/org_1/raw");
        write_basic_raw_export(&raw_dir, "cpu-main", "CPU Main", "prom-main");
        let common = make_common_args();

        let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: Some(repo_root),
            input_format: DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: None,
            queries: None,
            require_queries: false,
        })
        .unwrap();

        assert_eq!(artifacts.governance["summary"]["dashboardCount"], json!(1));
        assert_eq!(artifacts.queries["summary"]["dashboardCount"], json!(1));
        assert_eq!(
            artifacts.queries["queries"][0]["dashboardUid"],
            json!("cpu-main")
        );
    }

    #[test]
    fn resolve_dashboard_review_artifacts_ignores_permission_bundle_in_raw_export() {
        let temp = tempdir().unwrap();
        let raw_dir = temp.path().join("raw");
        write_basic_raw_export(&raw_dir, "cpu-main", "CPU Main", "prom-main");
        fs::write(
            raw_dir.join("permissions.json"),
            serde_json::to_string_pretty(&json!({
                "kind": "grafana-utils-dashboard-permission-bundle",
                "schemaVersion": 1,
                "summary": {
                    "resourceCount": 1,
                    "dashboardCount": 1,
                    "folderCount": 0,
                    "permissionCount": 1
                },
                "resources": [{
                    "kind": "grafana-utils-dashboard-permission-export",
                    "schemaVersion": 1,
                    "resourceKind": "dashboard",
                    "resource": {
                        "uid": "cpu-main",
                        "title": "CPU Main"
                    },
                    "permissions": [{
                        "subjectType": "role",
                        "subjectName": "Viewer",
                        "permission": 1,
                        "permissionName": "view"
                    }]
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        let common = make_common_args();

        let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: Some(&raw_dir),
            input_format: DashboardImportInputFormat::Raw,
            input_type: Some(InspectExportInputType::Raw),
            governance: None,
            queries: None,
            require_queries: false,
        })
        .unwrap();

        assert_eq!(artifacts.governance["summary"]["dashboardCount"], json!(1));
        assert_eq!(artifacts.queries["summary"]["dashboardCount"], json!(1));
        assert_eq!(artifacts.queries["summary"]["queryRecordCount"], json!(1));
    }

    #[test]
    fn resolve_dashboard_review_artifacts_requires_queries_for_gate_artifacts() {
        let temp = tempdir().unwrap();
        let governance_path = temp.path().join("governance.json");
        fs::write(
            &governance_path,
            serde_json::to_string_pretty(&json!({"summary": {"dashboardCount": 1}})).unwrap(),
        )
        .unwrap();
        let common = make_common_args();

        let error = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
            common: &common,
            page_size: 500,
            org_id: None,
            all_orgs: false,
            input_dir: None,
            input_format: DashboardImportInputFormat::Raw,
            input_type: None,
            governance: Some(&governance_path),
            queries: None,
            require_queries: true,
        })
        .unwrap_err();

        assert!(error.to_string().contains(
            "--queries is required when reusing saved dashboard review artifacts for policy"
        ));
    }
}
