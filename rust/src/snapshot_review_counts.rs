use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::common::Result;

#[derive(Debug, Clone, Default)]
pub(super) struct SnapshotReviewOrgCounts {
    pub(super) org: String,
    pub(super) org_id: String,
    pub(super) dashboard_count: usize,
    pub(super) folder_count: usize,
    pub(super) datasource_count: usize,
    pub(super) default_datasource_count: usize,
    pub(super) datasource_types: BTreeMap<String, usize>,
}

fn snapshot_review_org_key(org_id: &str, org: &str) -> String {
    if !org_id.trim().is_empty() {
        format!("org-id:{org_id}")
    } else if !org.trim().is_empty() {
        format!("org:{org}")
    } else {
        "org:unknown".to_string()
    }
}

pub(super) fn collect_dashboard_org_counts(
    dashboard_metadata: &Value,
    dashboard_index: &Value,
) -> Result<(Vec<SnapshotReviewOrgCounts>, usize, bool)> {
    let mut rows = Vec::new();
    let mut missing_org_scope = false;
    if let Some(orgs) = dashboard_metadata.get("orgs").and_then(Value::as_array) {
        for org in orgs {
            let org = org.as_object().ok_or_else(|| {
                crate::common::message("Snapshot dashboard export org entry must be a JSON object.")
            })?;
            let org_name = org
                .get("org")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let org_id = org
                .get("orgId")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if org_name.is_empty() && org_id.is_empty() {
                missing_org_scope = true;
            }
            rows.push(SnapshotReviewOrgCounts {
                org: org_name,
                org_id,
                dashboard_count: org
                    .get("dashboardCount")
                    .and_then(Value::as_u64)
                    .unwrap_or_default() as usize,
                folder_count: 0,
                datasource_count: 0,
                default_datasource_count: 0,
                datasource_types: BTreeMap::new(),
            });
        }
    } else {
        let org = dashboard_metadata
            .get("org")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let org_id = dashboard_metadata
            .get("orgId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if org.is_empty() && org_id.is_empty() {
            missing_org_scope = true;
        }
        rows.push(SnapshotReviewOrgCounts {
            org,
            org_id,
            dashboard_count: dashboard_metadata
                .get("dashboardCount")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize,
            folder_count: 0,
            datasource_count: 0,
            default_datasource_count: 0,
            datasource_types: BTreeMap::new(),
        });
    }

    if let Some(folders) = dashboard_index.get("folders").and_then(Value::as_array) {
        for folder in folders {
            let folder = folder.as_object().ok_or_else(|| {
                crate::common::message("Snapshot dashboard folder entry must be a JSON object.")
            })?;
            let org = folder
                .get("org")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let org_id = folder
                .get("orgId")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let key = snapshot_review_org_key(&org_id, &org);
            if let Some(entry) = rows
                .iter_mut()
                .find(|entry| snapshot_review_org_key(&entry.org_id, &entry.org) == key)
            {
                entry.folder_count += 1;
            }
        }
    }

    let dashboard_count = dashboard_metadata
        .get("dashboardCount")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            rows.iter()
                .map(|row| row.dashboard_count as u64)
                .sum::<u64>()
        }) as usize;
    Ok((rows, dashboard_count, missing_org_scope))
}

pub(super) fn collect_datasource_org_counts(
    datasource_rows: &[Value],
) -> Result<(Vec<SnapshotReviewOrgCounts>, usize, bool)> {
    let mut rows = BTreeMap::<String, SnapshotReviewOrgCounts>::new();
    let mut missing_org_scope = false;
    for datasource in datasource_rows {
        let datasource = datasource.as_object().ok_or_else(|| {
            crate::common::message("Snapshot datasource inventory entry must be a JSON object.")
        })?;
        let org = datasource
            .get("org")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let org_id = datasource
            .get("orgId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if org.is_empty() && org_id.is_empty() {
            missing_org_scope = true;
        }
        let key = snapshot_review_org_key(&org_id, &org);
        let entry = rows.entry(key).or_insert_with(|| SnapshotReviewOrgCounts {
            org: org.clone(),
            org_id: org_id.clone(),
            dashboard_count: 0,
            folder_count: 0,
            datasource_count: 0,
            default_datasource_count: 0,
            datasource_types: BTreeMap::new(),
        });
        if entry.org.is_empty() && !org.is_empty() {
            entry.org = org.clone();
        }
        if entry.org_id.is_empty() && !org_id.is_empty() {
            entry.org_id = org_id.clone();
        }
        entry.datasource_count += 1;
        if datasource
            .get("isDefault")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| {
                datasource
                    .get("isDefault")
                    .and_then(Value::as_str)
                    .map(|value| value == "true")
                    .unwrap_or(false)
            })
        {
            entry.default_datasource_count += 1;
        }
        let datasource_type = datasource
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        if !datasource_type.is_empty() {
            *entry
                .datasource_types
                .entry(datasource_type.to_string())
                .or_insert(0) += 1;
        }
    }
    Ok((
        rows.into_values().collect(),
        datasource_rows.len(),
        missing_org_scope,
    ))
}

pub(super) fn merge_snapshot_review_org_counts(
    dashboard_rows: Vec<SnapshotReviewOrgCounts>,
    datasource_rows: Vec<SnapshotReviewOrgCounts>,
) -> Vec<SnapshotReviewOrgCounts> {
    let mut orgs = BTreeMap::<String, SnapshotReviewOrgCounts>::new();
    for row in dashboard_rows {
        let key = snapshot_review_org_key(&row.org_id, &row.org);
        let entry = orgs.entry(key).or_default();
        if entry.org.is_empty() {
            entry.org = row.org.clone();
        }
        if entry.org_id.is_empty() {
            entry.org_id = row.org_id.clone();
        }
        entry.dashboard_count += row.dashboard_count;
        entry.folder_count += row.folder_count;
    }
    for row in datasource_rows {
        let key = snapshot_review_org_key(&row.org_id, &row.org);
        let entry = orgs.entry(key).or_default();
        if entry.org.is_empty() {
            entry.org = row.org.clone();
        }
        if entry.org_id.is_empty() {
            entry.org_id = row.org_id.clone();
        }
        entry.datasource_count += row.datasource_count;
        entry.default_datasource_count += row.default_datasource_count;
        for (datasource_type, count) in row.datasource_types {
            *entry.datasource_types.entry(datasource_type).or_insert(0) += count;
        }
    }
    orgs.into_values().collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_snapshot_review_warnings(
    dashboard_lane_summary: &Value,
    datasource_lane_summary: &Value,
    dashboard_org_count: usize,
    datasource_org_count: usize,
    dashboard_count: usize,
    datasource_count: usize,
    orgs: &[SnapshotReviewOrgCounts],
    missing_dashboard_org_scope: bool,
    missing_datasource_org_scope: bool,
) -> Vec<Value> {
    let mut warnings = Vec::new();
    if dashboard_org_count != datasource_org_count {
        warnings.push(json!({
            "code": "org-count-mismatch",
            "message": format!(
                "Dashboard export covers {} org(s) while datasource inventory covers {} org(s).",
                dashboard_org_count,
                datasource_org_count
            )
        }));
    }
    if dashboard_count == 0 {
        warnings.push(json!({
            "code": "empty-dashboard-inventory",
            "message": "Dashboard export did not record any dashboards."
        }));
    }
    if datasource_count == 0 {
        warnings.push(json!({
            "code": "empty-datasource-inventory",
            "message": "Datasource inventory did not record any datasources."
        }));
    }
    if missing_dashboard_org_scope {
        warnings.push(json!({
            "code": "dashboard-org-missing-scope",
            "message": "At least one dashboard export org entry is missing org or orgId metadata."
        }));
    }
    if missing_datasource_org_scope {
        warnings.push(json!({
            "code": "datasource-org-missing-scope",
            "message": "At least one datasource inventory row is missing org or orgId metadata."
        }));
    }
    let dashboard_scope_count = dashboard_lane_summary
        .get("scopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if dashboard_lane_summary
        .get("rawScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        < dashboard_scope_count
    {
        warnings.push(json!({
            "code": "dashboard-raw-lane-missing",
            "message": "At least one dashboard export scope is missing raw/ artifacts."
        }));
    }
    if dashboard_lane_summary
        .get("promptScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        < dashboard_scope_count
    {
        warnings.push(json!({
            "code": "dashboard-prompt-lane-missing",
            "message": "At least one dashboard export scope is missing prompt/ artifacts."
        }));
    }
    if dashboard_lane_summary
        .get("provisioningScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        < dashboard_scope_count
    {
        warnings.push(json!({
            "code": "dashboard-provisioning-lane-missing",
            "message": "At least one dashboard export scope is missing provisioning/ artifacts."
        }));
    }
    let datasource_inventory_scope_count = datasource_lane_summary
        .get("inventoryExpectedScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if datasource_lane_summary
        .get("inventoryScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        < datasource_inventory_scope_count
    {
        warnings.push(json!({
            "code": "datasource-inventory-lane-missing",
            "message": "At least one datasource export scope is missing datasources.json."
        }));
    }
    let datasource_provisioning_scope_count = datasource_lane_summary
        .get("provisioningExpectedScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if datasource_lane_summary
        .get("provisioningScopeCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        < datasource_provisioning_scope_count
    {
        warnings.push(json!({
            "code": "datasource-provisioning-lane-missing",
            "message": "At least one datasource export scope is missing provisioning/datasources.yaml."
        }));
    }
    for org in orgs {
        if org.dashboard_count == 0 || org.datasource_count == 0 {
            warnings.push(json!({
                "code": "org-partial-coverage",
                "message": format!(
                    "Org {} (orgId={}) has {} dashboard(s) and {} datasource(s).",
                    if org.org.is_empty() { "unknown" } else { &org.org },
                    if org.org_id.is_empty() { "unknown" } else { &org.org_id },
                    org.dashboard_count,
                    org.datasource_count
                )
            }));
        }
    }
    warnings
}
