"""Interactive dashboard delete helpers."""

from typing import Any, Callable

from .delete_support import clone_args, normalize_folder_path


def run_dashboard_delete_interactive(
    args: Any,
    deps: dict[str, Any],
) -> int:
    """Prompt for delete selectors, preview the plan, and confirm execution."""
    input_reader: Callable[[str], str] = deps.get("input_reader") or input
    output_writer: Callable[[str], None] = deps.get("output_writer") or print
    is_tty = deps.get("is_tty")
    if is_tty is not None and not is_tty():
        raise deps["GrafanaError"](
            "Dashboard delete interactive mode requires a TTY."
        )

    working_args = clone_args(args)
    uid = str(getattr(working_args, "uid", "") or "").strip()
    path = normalize_folder_path(getattr(working_args, "path", None))
    if not uid and not path:
        mode = input_reader("Delete by uid or path? [uid/path]: ").strip().lower()
        if mode not in ("uid", "path"):
            raise deps["GrafanaError"]("Interactive dashboard delete expected uid or path.")
        if mode == "uid":
            working_args.uid = input_reader("Dashboard UID: ").strip()
        else:
            working_args.path = normalize_folder_path(input_reader("Folder path: "))
    if (
        not bool(getattr(working_args, "delete_folders", False))
        and str(getattr(working_args, "path", "") or "").strip()
    ):
        include_folders = (
            input_reader("Also delete matching folders? [y/N]: ").strip().lower()
        )
        if include_folders in ("y", "yes"):
            working_args.delete_folders = True

    deps["validate_delete_args"](working_args)
    client = deps["build_client"](working_args)
    if getattr(working_args, "org_id", None):
        client = client.with_org_id(getattr(working_args, "org_id"))
    plan = deps["build_delete_plan"](client, working_args)
    for line in deps["render_dashboard_delete_text"](plan):
        output_writer(line)
    if bool(getattr(working_args, "dry_run", False)):
        return 0
    confirm = input_reader("Execute live dashboard delete? [y/N]: ").strip().lower()
    if confirm not in ("y", "yes"):
        output_writer("Cancelled dashboard delete.")
        return 1
    client = deps["build_client"](working_args)
    if getattr(working_args, "org_id", None):
        client = client.with_org_id(getattr(working_args, "org_id"))
    deps["execute_delete_plan"](client, plan)
    for item in plan.get("dashboards", []):
        output_writer(deps["format_live_dashboard_delete_line"](item))
    for item in plan.get("folders", []):
        output_writer(deps["format_live_folder_delete_line"](item))
    return 0
