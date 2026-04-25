//! Governance rule evaluation and risk scoring helpers for dashboard-level controls.

mod apply;
mod findings;
mod policy;

pub(crate) use apply::evaluate_dashboard_governance_gate_violations;
