//! Dashboard governance gate command runner.
use crate::common::{message, render_json_value, Result};

#[cfg(test)]
use super::build_governance_gate_tui_items;
#[cfg(all(feature = "tui", not(test)))]
use super::tui::run_governance_gate_interactive;
use super::{evaluate_dashboard_governance_gate, render_dashboard_governance_gate_result};
use crate::dashboard::review_source::{
    resolve_dashboard_review_artifacts, DashboardReviewSourceArgs,
};
use crate::dashboard::{
    load_governance_policy, write_json_document, GovernanceGateArgs, GovernanceGateOutputFormat,
};
#[cfg(test)]
use crate::interactive_browser::run_interactive_browser;

pub(crate) fn run_dashboard_governance_gate(args: &GovernanceGateArgs) -> Result<()> {
    let policy = load_governance_policy(args)?;
    let artifacts = resolve_dashboard_review_artifacts(&DashboardReviewSourceArgs {
        common: &args.common,
        page_size: args.page_size,
        org_id: args.org_id,
        all_orgs: args.all_orgs,
        input_dir: args.input_dir.as_deref(),
        input_format: args.input_format,
        input_type: args.input_type,
        governance: args.governance.as_deref(),
        queries: args.queries.as_deref(),
        require_queries: true,
    })?;
    let result =
        evaluate_dashboard_governance_gate(&policy, &artifacts.governance, &artifacts.queries)?;

    if let Some(output_path) = args.json_output.as_ref() {
        write_json_document(&result, output_path)?;
    }
    if args.interactive {
        #[cfg(all(feature = "tui", not(test)))]
        {
            run_governance_gate_interactive(&result)?;
            return governance_gate_exit_result(result.ok);
        }
        #[cfg(test)]
        {
            let summary_lines = render_dashboard_governance_gate_result(&result)
                .lines()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();
            run_interactive_browser(
                "Dashboard Governance Gate",
                &summary_lines,
                &build_governance_gate_tui_items(&result, "all"),
            )?;
            return governance_gate_exit_result(result.ok);
        }
        #[cfg(not(feature = "tui"))]
        {
            return crate::dashboard::tui_not_built("policy --interactive");
        }
    }
    match args.output_format {
        GovernanceGateOutputFormat::Json => {
            println!("{}", render_json_value(&result)?);
        }
        GovernanceGateOutputFormat::Text => {
            println!("{}", render_dashboard_governance_gate_result(&result));
        }
    }
    governance_gate_exit_result(result.ok)
}

fn governance_gate_exit_result(ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message(
            "Dashboard governance gate reported policy violations.",
        ))
    }
}
