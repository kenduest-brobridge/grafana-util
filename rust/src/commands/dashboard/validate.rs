//! Strict dashboard export validation.
//! Validates raw dashboard export files for schema migration, legacy properties, and custom
//! plugin usage before GitOps-style import or sync.
mod rules;

use serde_json::Value;
use std::path::Path;

use crate::common::{emit_plain_output, message, render_json_value, Result};

use rules::{validate_dashboard_document, DashboardValidationIssue};

use super::{discover_dashboard_files, load_json_file, ValidateExportArgs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DashboardValidationResult {
    pub(crate) dashboard_count: usize,
    pub(crate) error_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) issues: Vec<DashboardValidationIssue>,
}

pub(crate) fn validate_dashboard_export_dir(
    input_dir: &Path,
    reject_custom_plugins: bool,
    reject_legacy_properties: bool,
    target_schema_version: Option<i64>,
) -> Result<DashboardValidationResult> {
    let mut issues = Vec::new();
    let files = discover_dashboard_files(input_dir)?;
    for dashboard_file in &files {
        let document = load_json_file(dashboard_file)?;
        issues.extend(validate_dashboard_document(
            &document,
            dashboard_file,
            reject_custom_plugins,
            reject_legacy_properties,
            target_schema_version,
        )?);
    }
    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == "warning")
        .count();
    Ok(DashboardValidationResult {
        dashboard_count: files.len(),
        error_count,
        warning_count,
        issues,
    })
}

fn validation_result_document(result: &DashboardValidationResult) -> Value {
    let issues = result
        .issues
        .iter()
        .map(|issue| {
            serde_json::json!({
                "severity": issue.severity,
                "code": issue.code,
                "file": issue.file,
                "dashboardUid": issue.dashboard_uid,
                "dashboardTitle": issue.dashboard_title,
                "message": issue.message,
            })
        })
        .collect::<Vec<Value>>();
    serde_json::json!({
        "kind": "grafana-utils-dashboard-validation",
        "schemaVersion": 1,
        "summary": {
            "dashboardCount": result.dashboard_count,
            "errorCount": result.error_count,
            "warningCount": result.warning_count,
        },
        "issues": issues,
    })
}

fn render_validation_result_text(result: &DashboardValidationResult) -> Vec<String> {
    let mut lines = vec![
        "Dashboard validation".to_string(),
        format!("Dashboards: {}", result.dashboard_count),
        format!("Errors: {}", result.error_count),
        format!("Warnings: {}", result.warning_count),
    ];
    if !result.issues.is_empty() {
        lines.push(String::new());
        lines.push("# Issues".to_string());
        for issue in &result.issues {
            lines.push(format!(
                "[{}] {} {} {}",
                issue.severity.to_uppercase(),
                issue.code,
                issue.dashboard_uid,
                issue.message
            ));
        }
    }
    lines
}

pub(crate) fn render_validation_result_json(result: &DashboardValidationResult) -> Result<String> {
    Ok(format!(
        "{}\n",
        render_json_value(&validation_result_document(result))?
    ))
}

pub(crate) fn run_dashboard_validate_export(args: &ValidateExportArgs) -> Result<()> {
    let input_dir = super::source_loader::load_dashboard_source(
        &args.input_dir,
        args.input_format,
        None,
        false,
    )?;
    let result = validate_dashboard_export_dir(
        &input_dir.input_dir,
        args.reject_custom_plugins,
        args.reject_legacy_properties,
        args.target_schema_version,
    )?;
    let output = match args.output_format {
        super::ValidationOutputFormat::Text => {
            format!("{}\n", render_validation_result_text(&result).join("\n"))
        }
        super::ValidationOutputFormat::Json => render_validation_result_json(&result)?,
    };
    emit_plain_output(&output, args.output_file.as_deref(), args.also_stdout)?;
    if result.error_count > 0 {
        return Err(message(format!(
            "Dashboard validation found {} blocking issue(s).",
            result.error_count
        )));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use crate::common::{set_json_color_choice, CliColorChoice};
    use crate::dashboard::{DashboardImportInputFormat, ValidationOutputFormat};
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    fn write_valid_dashboard(path: &Path, uid: &str, title: &str) {
        fs::write(
            path,
            serde_json::to_string_pretty(&json!({
                "uid": uid,
                "title": title,
                "schemaVersion": 39,
                "panels": []
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn run_dashboard_validate_export_supports_provisioning_root() {
        let temp = tempdir().unwrap();
        let provisioning_root = temp.path().join("provisioning");
        let dashboards_dir = provisioning_root.join("dashboards/team");
        fs::create_dir_all(&dashboards_dir).unwrap();
        write_valid_dashboard(
            &dashboards_dir.join("cpu-main.json"),
            "cpu-main",
            "CPU Main",
        );
        let output_file = temp.path().join("validation.json");

        run_dashboard_validate_export(&ValidateExportArgs {
            input_dir: provisioning_root,
            input_format: DashboardImportInputFormat::Provisioning,
            reject_custom_plugins: true,
            reject_legacy_properties: true,
            target_schema_version: Some(39),
            output_format: ValidationOutputFormat::Json,
            output_file: Some(output_file.clone()),
            also_stdout: false,
        })
        .unwrap();

        let report = fs::read_to_string(output_file).unwrap();
        assert!(report.contains("\"dashboardCount\": 1"));
        assert!(report.contains("\"errorCount\": 0"));
    }

    #[test]
    fn run_dashboard_validate_export_supports_provisioning_dashboards_dir() {
        let temp = tempdir().unwrap();
        let provisioning_root = temp.path().join("provisioning");
        let dashboards_dir = provisioning_root.join("dashboards");
        fs::create_dir_all(&dashboards_dir).unwrap();
        write_valid_dashboard(
            &dashboards_dir.join("ops-main.json"),
            "ops-main",
            "Ops Main",
        );
        let output_file = temp.path().join("validation.json");

        run_dashboard_validate_export(&ValidateExportArgs {
            input_dir: dashboards_dir,
            input_format: DashboardImportInputFormat::Provisioning,
            reject_custom_plugins: false,
            reject_legacy_properties: false,
            target_schema_version: None,
            output_format: ValidationOutputFormat::Json,
            output_file: Some(output_file.clone()),
            also_stdout: false,
        })
        .unwrap();

        let report = fs::read_to_string(output_file).unwrap();
        assert!(
            report.contains("\"dashboardUid\": \"ops-main\"") || report.contains("\"issues\": []")
        );
        assert!(report.contains("\"dashboardCount\": 1"));
    }

    #[test]
    fn validate_dashboard_export_surfaces_dashboard_v2_resource_as_warning() {
        let temp = tempdir().unwrap();
        let input_dir = temp.path().join("raw");
        fs::create_dir_all(&input_dir).unwrap();
        fs::write(
            input_dir.join("v2.json"),
            serde_json::to_string_pretty(&json!({
                "apiVersion": "dashboard.grafana.app/v2",
                "kind": "Dashboard",
                "metadata": {"name": "v2-main"},
                "spec": {
                    "title": "V2 Main",
                    "elements": {}
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let result = validate_dashboard_export_dir(&input_dir, false, false, None).unwrap();

        assert_eq!(result.dashboard_count, 1);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.issues[0].code, "unsupported-dashboard-v2-resource");
        assert_eq!(result.issues[0].dashboard_uid, "v2-main");
        assert_eq!(result.issues[0].dashboard_title, "V2 Main");
    }

    #[test]
    fn validate_dashboard_import_document_rejects_dashboard_v2_resource() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("v2.json");
        let document = json!({
            "apiVersion": "dashboard.grafana.app/v2",
            "kind": "Dashboard",
            "metadata": {"name": "v2-main"},
            "spec": {
                "title": "V2 Main",
                "elements": {}
            }
        });

        let error = validate_dashboard_import_document(&document, &file, false, None)
            .unwrap_err()
            .to_string();

        assert!(error.contains("Refusing dashboard import"));
        assert!(error.contains("dashboard v2 resources are not supported yet"));
    }

    #[test]
    fn run_dashboard_validate_export_supports_git_sync_repo_root_without_export_metadata() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path().join("grafana-oac-repo");
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let dashboards_dir = repo_root.join("dashboards/git-sync/raw/org_1/raw");
        fs::create_dir_all(&dashboards_dir).unwrap();
        write_valid_dashboard(
            &dashboards_dir.join("cpu-main.json"),
            "cpu-main",
            "CPU Main",
        );
        let output_file = temp.path().join("validation-git-sync.json");

        run_dashboard_validate_export(&ValidateExportArgs {
            input_dir: repo_root,
            input_format: DashboardImportInputFormat::Raw,
            reject_custom_plugins: true,
            reject_legacy_properties: true,
            target_schema_version: Some(39),
            output_format: ValidationOutputFormat::Json,
            output_file: Some(output_file.clone()),
            also_stdout: false,
        })
        .unwrap();

        let report = fs::read_to_string(output_file).unwrap();
        assert!(report.contains("\"dashboardCount\": 1"));
        assert!(report.contains("\"errorCount\": 0"));
    }

    #[test]
    fn run_dashboard_validate_export_supports_also_stdout_with_output_file() {
        let temp = tempdir().unwrap();
        let provisioning_root = temp.path().join("provisioning");
        let dashboards_dir = provisioning_root.join("dashboards/team");
        fs::create_dir_all(&dashboards_dir).unwrap();
        write_valid_dashboard(
            &dashboards_dir.join("cpu-main.json"),
            "cpu-main",
            "CPU Main",
        );
        let output_file = temp.path().join("validation.json");

        run_dashboard_validate_export(&ValidateExportArgs {
            input_dir: provisioning_root,
            input_format: DashboardImportInputFormat::Provisioning,
            reject_custom_plugins: true,
            reject_legacy_properties: true,
            target_schema_version: Some(39),
            output_format: ValidationOutputFormat::Json,
            output_file: Some(output_file.clone()),
            also_stdout: true,
        })
        .unwrap();

        let report = fs::read_to_string(output_file).unwrap();
        assert!(report.contains("\"dashboardCount\": 1"));
        assert!(report.contains("\"errorCount\": 0"));
    }

    #[test]
    fn run_dashboard_validate_export_keeps_output_file_plain_even_with_color_enabled() {
        let temp = tempdir().unwrap();
        let provisioning_root = temp.path().join("provisioning");
        let dashboards_dir = provisioning_root.join("dashboards");
        fs::create_dir_all(&dashboards_dir).unwrap();
        write_valid_dashboard(
            &dashboards_dir.join("ops-main.json"),
            "ops-main",
            "Ops Main",
        );
        let output_file = temp.path().join("validation.json");
        set_json_color_choice(CliColorChoice::Always);

        let result = run_dashboard_validate_export(&ValidateExportArgs {
            input_dir: dashboards_dir,
            input_format: DashboardImportInputFormat::Provisioning,
            reject_custom_plugins: false,
            reject_legacy_properties: false,
            target_schema_version: None,
            output_format: ValidationOutputFormat::Json,
            output_file: Some(output_file.clone()),
            also_stdout: false,
        });

        set_json_color_choice(CliColorChoice::Auto);
        result.unwrap();

        let report = fs::read_to_string(output_file).unwrap();
        assert!(!report.contains('\u{1b}'));
        let parsed: Value = serde_json::from_str(&report).unwrap();
        assert_eq!(parsed["summary"]["dashboardCount"], json!(1));
    }
}

pub(crate) fn validate_dashboard_import_document(
    document: &Value,
    file: &Path,
    strict: bool,
    target_schema_version: Option<i64>,
) -> Result<()> {
    if super::is_dashboard_v2_resource(document) {
        return Err(message(format!(
            "Refusing dashboard import because Grafana dashboard v2 resources are not supported yet in {}.",
            file.display()
        )));
    }
    let issues =
        validate_dashboard_document(document, file, strict, strict, target_schema_version)?;
    let blocking = issues
        .iter()
        .filter(|issue| issue.severity == "error")
        .collect::<Vec<_>>();
    if blocking.is_empty() {
        return Ok(());
    }
    let first = blocking[0];
    Err(message(format!(
        "Refusing dashboard import because strict schema validation failed in {}: {} ({})",
        first.file, first.message, first.code
    )))
}
