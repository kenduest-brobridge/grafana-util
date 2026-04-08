use chrono::{DateTime, SecondsFormat, Utc};

fn build_time_utc() -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds, 0))
        .unwrap_or_else(Utc::now)
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn main() {
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rustc-env=GRAFANA_UTIL_BUILD_TIME={}", build_time_utc());
}
