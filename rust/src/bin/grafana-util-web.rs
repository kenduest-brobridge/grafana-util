//! Local-only web workbench binary entrypoint.
#![cfg(feature = "web")]

use grafana_utils_rust::web::{run_web_server, WebServerArgs};

#[tokio::main]
async fn main() {
    let args = <WebServerArgs as clap::Parser>::parse();
    if let Err(error) = run_web_server(args).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
