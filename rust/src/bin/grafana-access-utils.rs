use grafana_utils_rust::access::{parse_cli_from, run_access_cli};

fn main() {
    if let Err(error) = run_access_cli(parse_cli_from(std::env::args_os())) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
