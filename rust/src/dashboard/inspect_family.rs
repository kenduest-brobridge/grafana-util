//! Shared datasource family normalization helpers for inspect and governance paths.

pub(crate) fn normalize_family_name(datasource_type: &str) -> String {
    let lowered = datasource_type.trim().to_ascii_lowercase();
    let normalized = lowered
        .strip_prefix("grafana-")
        .and_then(|value| value.strip_suffix("-datasource"))
        .unwrap_or_else(|| lowered.strip_suffix("-datasource").unwrap_or(&lowered));
    match normalized {
        "" => "unknown".to_string(),
        "influxdb" | "flux" => "flux".to_string(),
        "prometheus" => "prometheus".to_string(),
        "loki" => "loki".to_string(),
        "mysql" | "postgres" | "mssql" | "postgresql" => "sql".to_string(),
        "elasticsearch" | "opensearch" => "search".to_string(),
        "tempo" | "jaeger" | "zipkin" => "tracing".to_string(),
        value => value.to_string(),
    }
}
