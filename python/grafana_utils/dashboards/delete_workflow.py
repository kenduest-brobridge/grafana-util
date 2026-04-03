"""Dashboard delete workflow orchestration."""

from .delete_interactive import run_dashboard_delete_interactive


def run_delete_dashboards(args, deps):
    """Run dashboard delete in dry-run, live, or interactive mode."""
    deps["validate_delete_args"](args)
    if bool(getattr(args, "interactive", False)):
        return run_dashboard_delete_interactive(args, deps)

    client = deps["build_client"](args)
    if getattr(args, "org_id", None):
        client = client.with_org_id(getattr(args, "org_id"))
    plan = deps["build_delete_plan"](client, args)
    if bool(getattr(args, "dry_run", False)):
        if bool(getattr(args, "json", False)):
            print(deps["render_dashboard_delete_json"](plan))
            return 0
        if bool(getattr(args, "table", False)):
            for line in deps["render_dashboard_delete_table"](
                plan.get("records", []),
                include_header=not bool(getattr(args, "no_header", False)),
            ):
                print(line)
        else:
            for line in deps["render_dashboard_delete_text"](plan):
                print(line)
        return 0

    deps["execute_delete_plan"](client, plan)
    for item in plan.get("dashboards", []):
        print(deps["format_live_dashboard_delete_line"](item))
    for item in plan.get("folders", []):
        print(deps["format_live_folder_delete_line"](item))
    summary = plan.get("summary") or {}
    print(
        "Deleted %s dashboard(s)%s"
        % (
            int(summary.get("dashboardCount") or 0),
            (
                " and %s folder(s)" % int(summary.get("folderCount") or 0)
                if int(summary.get("folderCount") or 0) > 0
                else ""
            ),
        )
    )
    return 0
