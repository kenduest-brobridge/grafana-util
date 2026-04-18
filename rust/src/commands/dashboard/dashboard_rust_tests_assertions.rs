use super::test_support::{ExportInspectionQueryReport, ExportInspectionQueryRow};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn read_json_output_file(path: &Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    assert!(
        raw.ends_with('\n'),
        "expected output file {} to end with a newline",
        path.display()
    );
    serde_json::from_str(&raw).unwrap()
}

fn json_query_report_row<'a>(document: &'a Value, ref_id: &str) -> &'a Value {
    document["queries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["refId"] == Value::String(ref_id.to_string()))
        .unwrap()
}

pub(crate) fn assert_json_query_report_row_parity(
    export_document: &Value,
    live_document: &Value,
    ref_id: &str,
) {
    let export_row = json_query_report_row(export_document, ref_id);
    let live_row = json_query_report_row(live_document, ref_id);
    for field in [
        "org",
        "orgId",
        "dashboardUid",
        "dashboardTitle",
        "dashboardTags",
        "folderPath",
        "folderFullPath",
        "folderLevel",
        "folderUid",
        "parentFolderUid",
        "panelId",
        "panelTitle",
        "panelType",
        "panelTargetCount",
        "panelQueryCount",
        "panelDatasourceCount",
        "panelVariables",
        "refId",
        "datasource",
        "datasourceName",
        "datasourceUid",
        "datasourceType",
        "datasourceFamily",
        "queryField",
        "targetHidden",
        "targetDisabled",
        "queryVariables",
        "metrics",
        "functions",
        "measurements",
        "buckets",
        "query",
    ] {
        assert_eq!(
            export_row[field], live_row[field],
            "field={field}, refId={ref_id}"
        );
    }
}

pub(crate) fn normalize_governance_document_for_compare(document: &Value) -> Value {
    let mut normalized = document.clone();
    if let Some(rows) = normalized
        .get_mut("dashboardDependencies")
        .and_then(|value| value.as_array_mut())
    {
        for row in rows {
            if let Some(object) = row.as_object_mut() {
                object.remove("file");
            }
        }
    }
    normalized
}

pub(crate) fn normalize_queries_document_for_compare(document: &Value) -> Value {
    let mut normalized = document.clone();
    if let Some(rows) = normalized
        .get_mut("queries")
        .and_then(|value| value.as_array_mut())
    {
        for row in rows {
            if let Some(object) = row.as_object_mut() {
                object.remove("file");
                object.remove("datasourceOrg");
                object.remove("datasourceOrgId");
                object.remove("datasourceDatabase");
                object.remove("datasourceBucket");
                object.remove("datasourceOrganization");
                object.remove("datasourceIndexPattern");
            }
        }
    }
    normalized
}

pub(crate) fn assert_governance_documents_match(export_document: &Value, live_document: &Value) {
    assert_eq!(
        normalize_governance_document_for_compare(export_document),
        normalize_governance_document_for_compare(live_document)
    );
}

pub(crate) fn assert_all_orgs_export_live_documents_match(
    export_report_document: &Value,
    live_report_document: &Value,
    export_dependency_document: &Value,
    live_dependency_document: &Value,
    export_governance_document: &Value,
    live_governance_document: &Value,
) {
    assert_eq!(
        normalize_queries_document_for_compare(export_report_document),
        normalize_queries_document_for_compare(live_report_document)
    );
    assert_eq!(
        normalize_queries_document_for_compare(export_dependency_document),
        normalize_queries_document_for_compare(live_dependency_document)
    );
    assert_governance_documents_match(export_governance_document, live_governance_document);
}

pub(crate) fn export_query_row<'a>(
    report: &'a ExportInspectionQueryReport,
    dashboard_uid: &str,
) -> &'a ExportInspectionQueryRow {
    report
        .queries
        .iter()
        .find(|query| query.dashboard_uid == dashboard_uid)
        .unwrap()
}

#[derive(Clone, Copy, Default)]
pub(crate) struct CoreFamilyQueryRowExpectation<'a> {
    pub(crate) dashboard_uid: &'a str,
    pub(crate) dashboard_title: &'a str,
    pub(crate) panel_id: &'a str,
    pub(crate) panel_title: &'a str,
    pub(crate) panel_type: &'a str,
    pub(crate) ref_id: &'a str,
    pub(crate) datasource: &'a str,
    pub(crate) datasource_name: &'a str,
    pub(crate) datasource_uid: &'a str,
    pub(crate) datasource_type: &'a str,
    pub(crate) datasource_family: &'a str,
    pub(crate) query_field: &'a str,
    pub(crate) query_text: &'a str,
    pub(crate) folder_path: &'a str,
    pub(crate) folder_full_path: &'a str,
    pub(crate) folder_level: &'a str,
    pub(crate) folder_uid: &'a str,
    pub(crate) parent_folder_uid: &'a str,
    pub(crate) datasource_org: &'a str,
    pub(crate) datasource_org_id: &'a str,
    pub(crate) datasource_database: &'a str,
    pub(crate) datasource_bucket: &'a str,
    pub(crate) datasource_organization: &'a str,
    pub(crate) datasource_index_pattern: &'a str,
    pub(crate) metrics: &'a [&'a str],
    pub(crate) functions: &'a [&'a str],
    pub(crate) measurements: &'a [&'a str],
    pub(crate) buckets: &'a [&'a str],
}

pub(crate) fn assert_core_family_query_row(
    report: &ExportInspectionQueryReport,
    expected: CoreFamilyQueryRowExpectation<'_>,
) {
    let row = export_query_row(report, expected.dashboard_uid);
    if !expected.dashboard_uid.is_empty() {
        assert_eq!(row.dashboard_uid, expected.dashboard_uid);
    }
    if !expected.dashboard_title.is_empty() {
        assert_eq!(row.dashboard_title, expected.dashboard_title);
    }
    if !expected.panel_id.is_empty() {
        assert_eq!(row.panel_id, expected.panel_id);
    }
    if !expected.panel_title.is_empty() {
        assert_eq!(row.panel_title, expected.panel_title);
    }
    if !expected.panel_type.is_empty() {
        assert_eq!(row.panel_type, expected.panel_type);
    }
    if !expected.ref_id.is_empty() {
        assert_eq!(row.ref_id, expected.ref_id);
    }
    if !expected.datasource.is_empty() {
        assert_eq!(row.datasource, expected.datasource);
    }
    if !expected.datasource_name.is_empty() {
        assert_eq!(row.datasource_name, expected.datasource_name);
    }
    if !expected.datasource_uid.is_empty() {
        assert_eq!(row.datasource_uid, expected.datasource_uid);
    }
    if !expected.datasource_type.is_empty() {
        assert_eq!(row.datasource_type, expected.datasource_type);
    }
    if !expected.datasource_family.is_empty() {
        assert_eq!(row.datasource_family, expected.datasource_family);
    }
    if !expected.query_field.is_empty() {
        assert_eq!(row.query_field, expected.query_field);
    }
    if !expected.query_text.is_empty() {
        assert_eq!(row.query_text, expected.query_text);
    }
    if !expected.folder_path.is_empty() {
        assert_eq!(row.folder_path, expected.folder_path);
    }
    if !expected.folder_full_path.is_empty() {
        assert_eq!(row.folder_full_path, expected.folder_full_path);
    }
    if !expected.folder_level.is_empty() {
        assert_eq!(row.folder_level, expected.folder_level);
    }
    if !expected.folder_uid.is_empty() {
        assert_eq!(row.folder_uid, expected.folder_uid);
    }
    if !expected.parent_folder_uid.is_empty() {
        assert_eq!(row.parent_folder_uid, expected.parent_folder_uid);
    }
    if !expected.datasource_org.is_empty() {
        assert_eq!(row.datasource_org, expected.datasource_org);
    }
    if !expected.datasource_org_id.is_empty() {
        assert_eq!(row.datasource_org_id, expected.datasource_org_id);
    }
    if !expected.datasource_database.is_empty() {
        assert_eq!(row.datasource_database, expected.datasource_database);
    }
    if !expected.datasource_bucket.is_empty() {
        assert_eq!(row.datasource_bucket, expected.datasource_bucket);
    }
    if !expected.datasource_organization.is_empty() {
        assert_eq!(
            row.datasource_organization,
            expected.datasource_organization
        );
    }
    if !expected.datasource_index_pattern.is_empty() {
        assert_eq!(
            row.datasource_index_pattern,
            expected.datasource_index_pattern
        );
    }
    assert_eq!(row.dashboard_tags, Vec::<String>::new());
    assert_eq!(row.panel_target_count, 1);
    assert_eq!(row.panel_query_count, 1);
    assert_eq!(row.panel_datasource_count, 1);
    assert_eq!(row.panel_variables, Vec::<String>::new());
    assert_eq!(row.query_variables, Vec::<String>::new());
    assert_eq!(row.target_hidden, "false");
    assert_eq!(row.target_disabled, "false");
    assert_eq!(
        row.metrics,
        expected
            .metrics
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.functions,
        expected
            .functions
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.measurements,
        expected
            .measurements
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
    assert_eq!(
        row.buckets,
        expected
            .buckets
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
    );
}
