use chrono::{DateTime, SecondsFormat, Utc};
use std::process::Command;

fn build_time_utc() -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn git_commit_short() -> String {
    std::env::var("GRAFANA_UTIL_GIT_COMMIT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=GRAFANA_UTIL_GIT_COMMIT");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/packed-refs");
    println!("cargo:rustc-env=GRAFANA_UTIL_BUILD_TIME={}", build_time_utc());
    println!("cargo:rustc-env=GRAFANA_UTIL_GIT_COMMIT={}", git_commit_short());
}
