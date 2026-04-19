use crate::common::{message, Result};

fn validate_import_org_auth(
    context: &super::super::DashboardAuthContext,
    args: &super::super::ImportArgs,
) -> Result<()> {
    if args.org_id.is_some() && context.auth_mode != "basic" {
        return Err(message(
            "Dashboard import with --org-id requires Basic auth (--basic-user / --basic-password).",
        ));
    }
    if args.use_export_org && context.auth_mode != "basic" {
        return Err(message(
            "Dashboard import with --use-export-org requires Basic auth (--basic-user / --basic-password).",
        ));
    }
    Ok(())
}

/// Purpose: implementation note.
pub(crate) fn build_import_auth_context(
    args: &super::super::ImportArgs,
) -> Result<super::super::DashboardAuthContext> {
    let mut context = super::super::build_auth_context(&args.common)?;
    validate_import_org_auth(&context, args)?;
    if let Some(org_id) = args.org_id {
        context
            .headers
            .push(("X-Grafana-Org-Id".to_string(), org_id.to_string()));
    }
    Ok(context)
}
