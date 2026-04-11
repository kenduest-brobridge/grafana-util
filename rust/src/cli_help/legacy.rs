pub fn legacy_command_error_hint(args: &[String]) -> Option<&'static str> {
    let command = args.get(1).map(String::as_str)?;
    match command {
        "observe" => Some(
            "tip: 'observe' was removed. Use 'grafana-util status live' for the first read-only check, or 'grafana-util status overview live' for the live overview.",
        ),
        "overview" => Some(
            "tip: the top-level 'overview' root was removed. Use 'grafana-util status overview live' for live Grafana, or 'grafana-util status overview' with local artifact inputs.",
        ),
        "change" => Some(
            "tip: 'change' was renamed to 'workspace'. Use 'grafana-util workspace scan', 'workspace test', or 'workspace preview' before apply.",
        ),
        "advanced" if args.get(2).map(String::as_str) == Some("dashboard") => Some(
            "tip: 'advanced dashboard' was removed. Use the flat 'grafana-util dashboard ...' commands; start with 'dashboard --help' to choose browse, list, export, summary, diff, or policy.",
        ),
        "dashboard" if args.get(2).map(String::as_str) == Some("live") => Some(
            "tip: 'dashboard live' was removed. Use 'grafana-util status live' for a read-only Grafana check, or 'grafana-util dashboard browse|list' for dashboard inventory.",
        ),
        _ => None,
    }
}
