//! Import validation facade for dashboard import authentication, org routing, and preflight checks.

#[path = "validation_auth.rs"]
mod auth;
#[path = "validation_dependencies.rs"]
mod dependencies;
#[path = "validation_org_scope.rs"]
mod org_scope;

pub(crate) use auth::build_import_auth_context;
pub(crate) use dependencies::{
    validate_dashboard_import_dependencies_with_client,
    validate_dashboard_import_dependencies_with_request,
};
pub(crate) use org_scope::{
    discover_export_org_import_scopes, resolve_target_org_plan_for_export_scope_with_request,
    validate_matching_export_org_with_client, validate_matching_export_org_with_request,
    ExportOrgImportScope, ExportOrgTargetPlan,
};
