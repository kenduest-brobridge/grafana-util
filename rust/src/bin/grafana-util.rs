//! Unified Rust CLI binary entrypoint.
//!
//! Flow:
//! - Parse raw argv for the special `--help-full` pre-check path.
//! - Fall back to normal unified CLI parse and dispatch.
//! - Print any top-level error and exit with status 1.
use grafana_utils_rust::cli::{maybe_render_unified_help_from_os_args, parse_cli_from, run_cli};
use std::io::IsTerminal;

/// Binary entrypoint for the Rust unified CLI.
///
/// Resolution order:
/// 1) unified pre-flight help hooks (including dashboard extensions)
/// 2) normal parse + dispatch via `run_cli`
fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();

    // Fast path: render expanded unified help without entering the heavy dispatch path
    // when users request docs-oriented flags like `--help-full`.
    if let Some(help_text) =
        maybe_render_unified_help_from_os_args(args.clone(), std::io::stdout().is_terminal())
    {
        print!("{help_text}");
        return;
    }

    // Normal execution: parse once, then dispatch through the shared runner.
    // Hand off to the shared dispatcher; non-zero exit indicates command-level error.
    if let Err(error) = run_cli(parse_cli_from(args)) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
