//! Organization import/export/diff workflow helpers.
//!
//! Maintainer notes:
//! - Org workflows require Basic auth because they touch Grafana admin APIs and
//!   can create, rename, delete, or enumerate orgs outside the current context.
//! - Org import reconciles org existence and listed user roles, but it does not
//!   remove extra live org users that are absent from the export bundle.

#[path = "org_workflows_diff.rs"]
mod diff;
#[path = "org_workflows_live.rs"]
mod live;
#[path = "org_workflows_sync.rs"]
mod sync;

pub(crate) use self::diff::diff_orgs_with_request;
pub(crate) use self::live::{
    add_org_with_request, delete_org_with_request, list_orgs_from_input_dir,
    list_orgs_with_request, modify_org_with_request,
};
pub(crate) use self::sync::{export_orgs_with_request, import_orgs_with_request};
