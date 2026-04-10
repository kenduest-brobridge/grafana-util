//! Unified CLI help examples and rendering helpers.
//!
//! Keeping the large example blocks and help rendering here lets `cli.rs`
//! stay focused on command topology and dispatch.

use clap::{ColorChoice, CommandFactory};

use crate::access::root_command as access_root_command;
use crate::alert::root_command as alert_root_command;
use crate::cli::CliArgs;
use crate::cli_help_examples::{
    colorize_dashboard_short_help, colorize_dashboard_subcommand_help, colorize_help_examples,
    inject_help_full_hint, ACCESS_HELP_FULL_TEXT, ALERT_HELP_FULL_TEXT, DATASOURCE_HELP_FULL_TEXT,
    SYNC_HELP_FULL_TEXT, UNIFIED_HELP_FULL_TEXT, UNIFIED_HELP_TEXT,
};
use crate::datasource::root_command as datasource_root_command;
use crate::sync::SyncCliArgs;

pub(crate) const UNIFIED_DASHBOARD_HELP_TEXT: &str = "Examples:\n\n  grafana-util dashboard live browse --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\"\n  grafana-util dashboard live fetch --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --dashboard-uid cpu-main --output ./cpu-main.json\n  grafana-util dashboard live clone --url http://localhost:3000 --basic-user admin --basic-password admin --source-uid cpu-main --output ./cpu-main-clone.json\n  grafana-util dashboard analyze summary --url http://localhost:3000 --basic-user admin --basic-password admin --output-format governance-json\n  grafana-util dashboard sync export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./dashboards --overwrite --include-history\n  grafana-util dashboard sync diff --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw\n  grafana-util dashboard analyze topology --url http://localhost:3000 --basic-user admin --basic-password admin --output-format mermaid\n  grafana-util dashboard draft patch --input ./dashboards/raw/cpu-main.json --name 'CPU Overview' --folder-uid infra --tag prod --tag sre\n  grafana-util dashboard draft review --input ./drafts/cpu-main.json --output-format yaml\n  grafana-util dashboard draft publish --url http://localhost:3000 --basic-user admin --basic-password admin --input ./drafts/cpu-main.json --dry-run --table";
pub(crate) const UNIFIED_DASHBOARD_SHORT_HELP_TEXT: &str = "Usage: grafana-util dashboard <COMMAND>\n\nChoose the task first:\n  live         browse, list, vars, fetch, clone, edit, delete, history\n  draft        review, patch, serve, publish\n  sync         export, import, diff, convert\n  analyze      summary, topology, impact, governance\n  capture      screenshot\n\nMore help:\n  grafana-util dashboard <COMMAND> --help\n  grafana-util dashboard <COMMAND> --help-full\n";
pub(crate) const UNIFIED_DATASOURCE_HELP_TEXT: &str = "Examples:\n\n  grafana-util datasource browse --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\"\n  grafana-util datasource list --input-dir ./datasources --json\n  grafana-util datasource list --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --json\n  grafana-util datasource import --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --input-dir ./datasources --dry-run --json";
pub(crate) const UNIFIED_SYNC_HELP_TEXT: &str = "Examples:\n\n  grafana-util change inspect --workspace ./grafana-oac-repo --output-format table\n  grafana-util change preview --workspace ./grafana-oac-repo --fetch-live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --output-format json\n  grafana-util change apply --preview-file ./change-preview.json --approve --execute-live --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\"";
pub(crate) const UNIFIED_ALERT_HELP_TEXT: &str = "Examples:\n\n  grafana-util alert migrate export --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --output-dir ./alerts --overwrite\n  grafana-util alert migrate import --url http://localhost:3000 --input-dir ./alerts/raw --replace-existing --dry-run --json\n  grafana-util alert live list-rules --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --json";
pub(crate) const ALERT_SHORT_HELP_TEXT: &str = "Usage: grafana-util alert <COMMAND>\n\nChoose the task first:\n  live         list-rules, list-contact-points, list-mute-timings, list-templates, delete\n  migrate      export, import, diff\n  author       init, rule add|clone, contact-point add, route set|preview\n  scaffold     rule, contact-point, template\n  change       plan, apply\n\nMore help:\n  grafana-util alert <COMMAND> --help\n  grafana-util alert <COMMAND> --help-full\n";
pub(crate) const UNIFIED_ACCESS_HELP_TEXT: &str = "Examples:\n\n  grafana-util access user list --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --json\n  grafana-util access user list --input-dir ./access-users --json\n  grafana-util access team import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./access-teams --replace-existing --yes\n  grafana-util access service-account token add --url http://localhost:3000 --token \"$GRAFANA_API_TOKEN\" --name deploy-bot --token-name nightly";

const DASHBOARD_DIFF_SCHEMA_HELP_TEXT: &str = include_str!("../../schemas/help/diff/dashboard.txt");
const ALERT_DIFF_SCHEMA_HELP_TEXT: &str = include_str!("../../schemas/help/diff/alert.txt");
const DATASOURCE_DIFF_SCHEMA_HELP_TEXT: &str =
    include_str!("../../schemas/help/diff/datasource.txt");

fn ensure_trailing_blank_line(mut text: String) -> String {
    if text.ends_with("\n\n") {
        return text;
    }
    if text.ends_with('\n') {
        text.push('\n');
    } else {
        text.push_str("\n\n");
    }
    text
}

fn render_long_help_with_color_choice(command: &mut clap::Command, colorize: bool) -> String {
    let configured = std::mem::take(command)
        .styles(crate::help_styles::CLI_HELP_STYLES)
        .color(if colorize {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        });
    *command = configured;
    let rendered = command.render_long_help();
    if colorize {
        rendered.ansi().to_string()
    } else {
        rendered.to_string()
    }
}

fn render_domain_help_text(mut command: clap::Command, colorize: bool) -> String {
    ensure_trailing_blank_line(inject_help_full_hint(render_long_help_with_color_choice(
        &mut command,
        colorize,
    )))
}

fn render_domain_help_full_text(
    mut command: clap::Command,
    extended_examples: &str,
    colorize: bool,
) -> String {
    let mut help = render_long_help_with_color_choice(&mut command, colorize);
    if colorize {
        help.push_str(&colorize_help_examples(extended_examples));
    } else {
        help.push_str(extended_examples);
    }
    ensure_trailing_blank_line(help)
}

fn render_unified_subcommand_help(path: &[String], colorize: bool) -> Option<String> {
    let mut command = CliArgs::command();
    let mut current = &mut command;
    for segment in path {
        current = current.find_subcommand_mut(segment)?;
    }
    let help = render_long_help_with_color_choice(current, colorize);
    let usage_prefix = format!("Usage: {}", path.last()?);
    let help = help.replacen(
        &usage_prefix,
        &format!("Usage: grafana-util {}", path.join(" ")),
        1,
    );
    if !colorize {
        return Some(ensure_trailing_blank_line(help));
    }
    let help = if path.iter().any(|segment| segment == "dashboard") {
        colorize_dashboard_subcommand_help(&help)
    } else {
        colorize_help_examples(&help)
    };
    Some(ensure_trailing_blank_line(help))
}

pub fn render_unified_help_text(colorize: bool) -> String {
    let mut command = CliArgs::command();
    let help = inject_help_full_hint(render_long_help_with_color_choice(&mut command, colorize));
    if colorize {
        ensure_trailing_blank_line(help.replace(
            UNIFIED_HELP_TEXT,
            &colorize_help_examples(UNIFIED_HELP_TEXT),
        ))
    } else {
        ensure_trailing_blank_line(help)
    }
}

pub fn render_unified_help_full_text(colorize: bool) -> String {
    let mut help = render_unified_help_text(colorize);
    if colorize {
        help.push_str(&colorize_help_examples(UNIFIED_HELP_FULL_TEXT));
    } else {
        help.push_str(UNIFIED_HELP_FULL_TEXT);
    }
    ensure_trailing_blank_line(help)
}

pub fn render_unified_version_text() -> String {
    crate::common::TOOL_VERSION_TEXT.to_string()
}

fn render_change_schema_help(target: Option<&str>) -> Option<String> {
    match target {
        None => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/root.help.txt"
            ))
            .to_string(),
        ),
        Some("summary") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/summary.help.txt"
            ))
            .to_string(),
        ),
        Some("plan") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/plan.help.txt"
            ))
            .to_string(),
        ),
        Some("review") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/review.help.txt"
            ))
            .to_string(),
        ),
        Some("apply") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/apply.help.txt"
            ))
            .to_string(),
        ),
        Some("audit") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/audit.help.txt"
            ))
            .to_string(),
        ),
        Some("preflight") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/preflight.help.txt"
            ))
            .to_string(),
        ),
        Some("assess-alerts") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/assess-alerts.help.txt"
            ))
            .to_string(),
        ),
        Some("bundle-preflight") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/bundle-preflight.help.txt"
            ))
            .to_string(),
        ),
        Some("promotion-preflight") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/promotion-preflight.help.txt"
            ))
            .to_string(),
        ),
        Some("bundle") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/change/bundle.help.txt"
            ))
            .to_string(),
        ),
        _ => None,
    }
}

fn render_dashboard_history_schema_help(target: Option<&str>) -> Option<String> {
    match target {
        None => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/dashboard-history/root.help.txt"
            ))
            .to_string(),
        ),
        Some("list") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/dashboard-history/list.help.txt"
            ))
            .to_string(),
        ),
        Some("restore") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/dashboard-history/restore.help.txt"
            ))
            .to_string(),
        ),
        Some("diff") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/dashboard-history/diff.help.txt"
            ))
            .to_string(),
        ),
        Some("export") => Some(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../schemas/help/dashboard-history/export.help.txt"
            ))
            .to_string(),
        ),
        _ => None,
    }
}

fn render_diff_schema_help(domain: &str) -> Option<String> {
    match domain {
        "dashboard" => Some(DASHBOARD_DIFF_SCHEMA_HELP_TEXT.to_string()),
        "alert" => Some(ALERT_DIFF_SCHEMA_HELP_TEXT.to_string()),
        "datasource" => Some(DATASOURCE_DIFF_SCHEMA_HELP_TEXT.to_string()),
        _ => None,
    }
}

fn render_status_schema_help(target: Option<&str>) -> Option<String> {
    match target {
        None => Some(include_str!("../../schemas/help/status/root.txt").to_string()),
        Some("staged") => Some(include_str!("../../schemas/help/status/staged.txt").to_string()),
        Some("live") => Some(include_str!("../../schemas/help/status/live.txt").to_string()),
        _ => None,
    }
}

pub fn maybe_render_unified_help_from_os_args<I, T>(iter: I, colorize: bool) -> Option<String>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args = iter
        .into_iter()
        .map(|value| value.into().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if args.len() >= 3
        && args.get(1).map(String::as_str) == Some("change")
        && args.iter().any(|value| value == "--help-schema")
    {
        let target = args
            .get(2)
            .filter(|value| !value.starts_with('-'))
            .map(String::as_str);
        return render_change_schema_help(target);
    }
    if args.len() >= 4
        && args.get(1).map(String::as_str) == Some("dashboard")
        && args.get(2).map(String::as_str) == Some("history")
        && args.iter().any(|value| value == "--help-schema")
    {
        let target = args
            .get(3)
            .filter(|value| !value.starts_with('-'))
            .map(String::as_str);
        return render_dashboard_history_schema_help(target);
    }
    if args.len() >= 4
        && args.get(1).map(String::as_str) == Some("dashboard")
        && args.get(2).map(String::as_str) == Some("diff")
        && args.iter().any(|value| value == "--help-schema")
    {
        return render_diff_schema_help("dashboard");
    }
    if args.len() >= 4
        && args.get(1).map(String::as_str) == Some("alert")
        && args.get(2).map(String::as_str) == Some("diff")
        && args.iter().any(|value| value == "--help-schema")
    {
        return render_diff_schema_help("alert");
    }
    if args.len() >= 4
        && args.get(1).map(String::as_str) == Some("datasource")
        && args.get(2).map(String::as_str) == Some("diff")
        && args.iter().any(|value| value == "--help-schema")
    {
        return render_diff_schema_help("datasource");
    }
    if args.len() >= 3
        && matches!(args.get(1).map(String::as_str), Some("status" | "observe"))
        && args.iter().any(|value| value == "--help-schema")
    {
        let target = args
            .get(2)
            .filter(|value| !value.starts_with('-'))
            .map(String::as_str);
        return render_status_schema_help(target);
    }
    if let Some(help_index) = args
        .iter()
        .position(|value| value == "--help" || value == "-h")
    {
        if help_index >= 2 {
            let path = args[1..help_index].to_vec();
            if matches!(
                path.first().map(String::as_str),
                Some("advanced") | Some("export")
            ) {
                return render_unified_subcommand_help(&path, colorize);
            }
        }
    }
    match args.as_slice() {
        [_binary] => Some(render_unified_help_text(colorize)),
        [_binary, flag] if flag == "--help" || flag == "-h" => {
            Some(render_unified_help_text(colorize))
        }
        [_binary, flag] if flag == "--help-full" => Some(render_unified_help_full_text(colorize)),
        [_binary, command, flag]
            if command == "datasource" && (flag == "--help" || flag == "-h") =>
        {
            Some(render_domain_help_text(datasource_root_command(), colorize))
        }
        [_binary, command, flag] if command == "access" && (flag == "--help" || flag == "-h") => {
            Some(render_domain_help_text(access_root_command(), colorize))
        }
        [_binary, command, flag] if command == "change" && (flag == "--help" || flag == "-h") => {
            Some(render_domain_help_text(SyncCliArgs::command(), colorize))
        }
        [_binary, command, flag]
            if command == "dashboard" && (flag == "--help" || flag == "-h") =>
        {
            Some(if colorize {
                colorize_dashboard_short_help(UNIFIED_DASHBOARD_SHORT_HELP_TEXT)
            } else {
                UNIFIED_DASHBOARD_SHORT_HELP_TEXT.to_string()
            })
        }
        [_binary, command, flag] if command == "alert" && (flag == "--help" || flag == "-h") => {
            Some(ALERT_SHORT_HELP_TEXT.to_string())
        }
        [_binary, command, flag] if command == "alert" && flag == "--help-full" => Some(
            render_domain_help_full_text(alert_root_command(), ALERT_HELP_FULL_TEXT, colorize),
        ),
        [_binary, command, flag] if command == "datasource" && flag == "--help-full" => {
            Some(render_domain_help_full_text(
                datasource_root_command(),
                DATASOURCE_HELP_FULL_TEXT,
                colorize,
            ))
        }
        [_binary, command, flag] if command == "access" && flag == "--help-full" => Some(
            render_domain_help_full_text(access_root_command(), ACCESS_HELP_FULL_TEXT, colorize),
        ),
        [_binary, command, flag] if command == "change" && flag == "--help-full" => Some(
            render_domain_help_full_text(SyncCliArgs::command(), SYNC_HELP_FULL_TEXT, colorize),
        ),
        _ => None,
    }
}
