use grafana_utils_rust::alert::{parse_cli_from, run_alert_cli};

fn main() {
    if let Err(error) = run_alert_cli(parse_cli_from(std::env::args_os())) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
