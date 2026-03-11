use grafana_utils_rust::dashboard::{parse_cli_from, run_dashboard_cli};

fn main() {
    if let Err(error) = run_dashboard_cli(parse_cli_from(std::env::args_os())) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
