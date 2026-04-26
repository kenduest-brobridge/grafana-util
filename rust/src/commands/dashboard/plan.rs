//! Dashboard review-first plan builder and renderer.

mod input;
mod permissions;
mod reconcile;
mod render;

use serde_json::Value;

use crate::common::{message, print_supported_columns, tool_version, Result};

use self::input::collect_plan_input;
use self::permissions::build_folder_permission_actions;
use self::reconcile::{build_org_actions, build_org_summary, build_summary};
pub(crate) use self::render::dashboard_plan_column_ids;
use self::render::print_dashboard_plan_report;
use super::plan_types as types;
use types::{DashboardPlanInput, DashboardPlanReport};

#[cfg(test)]
use self::input::collect_plan_input_with_request;
#[cfg(test)]
use self::reconcile::build_local_dashboard;
#[cfg(test)]
use self::render::{render_plan_table, render_plan_text};
#[cfg(test)]
use super::FolderInventoryItem;
#[cfg(test)]
use types::{
    FolderPermissionEntry, FolderPermissionResource, LiveDashboard, LocalDashboard, OrgPlanInput,
};

const PLAN_KIND: &str = "grafana-util-dashboard-plan";
const PLAN_SCHEMA_VERSION: i64 = 1;

pub(crate) fn build_dashboard_plan(input: DashboardPlanInput) -> DashboardPlanReport {
    let mut orgs = Vec::new();
    let mut actions = Vec::new();
    for org in &input.orgs {
        let mut org_actions = build_org_actions(org, input.prune);
        if input.include_folder_permissions {
            org_actions.extend(build_folder_permission_actions(
                org,
                &input.folder_permission_match,
            ));
        }
        orgs.push(build_org_summary(org, &org_actions));
        actions.extend(org_actions);
    }
    let summary = build_summary(&orgs, &actions);
    let mut report = DashboardPlanReport {
        kind: PLAN_KIND.to_string(),
        schema_version: PLAN_SCHEMA_VERSION,
        tool_version: tool_version().to_string(),
        mode: "review".to_string(),
        scope: input.scope,
        input_type: input.input_type,
        input_layout: input.input_layout,
        prune: input.prune,
        summary,
        review: Value::Null,
        orgs,
        actions,
    };
    report.review = report.build_review_envelope();
    report
}

pub(crate) fn build_dashboard_plan_json(report: &DashboardPlanReport) -> Result<Value> {
    serde_json::to_value(report).map_err(|error| message(error.to_string()))
}

pub(crate) fn run_dashboard_plan(args: &super::PlanArgs) -> Result<usize> {
    if !args.output_columns.is_empty()
        && args.output_format != super::DashboardPlanOutputFormat::Table
    {
        return Err(message(
            "--output-columns is only supported with --output-format table for dashboard plan.",
        ));
    }
    if args.no_header && args.output_format != super::DashboardPlanOutputFormat::Table {
        return Err(message(
            "--no-header is only supported with --output-format table for dashboard plan.",
        ));
    }
    if args.list_columns {
        print_supported_columns(dashboard_plan_column_ids());
        return Ok(0);
    }
    let input = collect_plan_input(args)?;
    let report = build_dashboard_plan(input);
    print_dashboard_plan_report(
        &report,
        args.output_format,
        args.show_same,
        args.no_header,
        &args.output_columns,
    )?;
    Ok(report.summary.checked)
}

#[cfg(test)]
#[path = "dashboard_plan_rust_tests.rs"]
mod dashboard_plan_rust_tests;
