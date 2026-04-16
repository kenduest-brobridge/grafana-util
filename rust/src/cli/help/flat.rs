use clap::{Command, CommandFactory};

use super::contextual::ensure_trailing_blank_line;
use crate::cli::CliArgs;

struct FlatHelpRow {
    command: String,
}

fn collect_flat_help_rows(command: &Command, path: &mut Vec<String>, rows: &mut Vec<FlatHelpRow>) {
    let visible_subcommands = command
        .get_subcommands()
        .filter(|subcommand| !subcommand.is_hide_set())
        .collect::<Vec<_>>();
    if !path.is_empty() {
        rows.push(FlatHelpRow {
            command: format!("grafana-util {}", path.join(" ")),
        });
    }
    for subcommand in visible_subcommands {
        path.push(subcommand.get_name().to_string());
        collect_flat_help_rows(subcommand, path, rows);
        path.pop();
    }
}

fn render_flat_help_table(rows: &[FlatHelpRow], colorize: bool) -> String {
    let mut lines = vec![
        "Flat command inventory".to_string(),
        "One public command path per line. Use `grafana-util <COMMAND> --help` for purpose and flags.".to_string(),
        String::new(),
        "COMMAND".to_string(),
    ];
    for row in rows {
        let command = if colorize {
            crate::cli_help_examples::HELP_PALETTE.command.to_string()
                + &row.command
                + crate::cli_help_examples::HELP_PALETTE.reset
        } else {
            row.command.clone()
        };
        lines.push(command);
    }
    lines.join("\n")
}

pub fn render_unified_help_flat_text(colorize: bool) -> String {
    let command = CliArgs::command();
    let mut rows = Vec::new();
    collect_flat_help_rows(&command, &mut Vec::new(), &mut rows);
    ensure_trailing_blank_line(render_flat_help_table(&rows, colorize))
}
