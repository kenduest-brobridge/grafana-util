//! Datasource type catalog shared by datasource CLI surfaces.
//!
//! Purpose:
//! - Keep supported datasource categories and type ids centralized.
//! - Provide one stable scaffold for future datasource-specific validation and presets.

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

use clap::ValueEnum;

/// One supported datasource type entry grouped under a higher-level category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatasourceCatalogEntry {
    pub category: &'static str,
    pub type_id: &'static str,
    pub display_name: &'static str,
    pub aliases: &'static [&'static str],
    pub profile: &'static str,
    pub query_language: &'static str,
    pub suggested_flags: &'static [&'static str],
    pub target: &'static str,
    pub requires_url: bool,
    pub url_example: &'static str,
    pub add_defaults_access: Option<&'static str>,
    pub add_defaults_http_method: Option<&'static str>,
    pub add_defaults_time_field: Option<&'static str>,
    pub add_defaults_json_data: &'static [(&'static str, DatasourceCatalogJsonDefaultValue)],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DatasourcePresetProfile {
    Starter,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasourceCatalogJsonDefaultValue {
    Bool(bool),
    Number(i64),
    String(&'static str),
}

impl DatasourceCatalogJsonDefaultValue {
    fn to_json_value(self) -> Value {
        match self {
            Self::Bool(value) => Value::Bool(value),
            Self::Number(value) => Value::Number(value.into()),
            Self::String(value) => Value::String(value.to_string()),
        }
    }

    fn to_display_value(self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => value.to_string(),
            Self::String(value) => value.to_string(),
        }
    }
}

const DATASOURCE_CATALOG: &[DatasourceCatalogEntry] = &[
    DatasourceCatalogEntry {
        category: "Metrics",
        type_id: "prometheus",
        display_name: "Prometheus",
        aliases: &["grafana-prometheus-datasource"],
        profile: "metrics-http",
        query_language: "promql",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://prometheus:9090",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: Some("POST"),
        add_defaults_time_field: None,
        add_defaults_json_data: &[],
    },
    DatasourceCatalogEntry {
        category: "Metrics",
        type_id: "influxdb",
        display_name: "InfluxDB",
        aliases: &["grafana-influxdb-datasource", "flux"],
        profile: "metrics-http",
        query_language: "flux-or-influxql",
        suggested_flags: &[
            "--user",
            "--password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://influxdb:8086",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            ("version", DatasourceCatalogJsonDefaultValue::String("Flux")),
            (
                "organization",
                DatasourceCatalogJsonDefaultValue::String("main-org"),
            ),
            (
                "defaultBucket",
                DatasourceCatalogJsonDefaultValue::String("metrics"),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Metrics",
        type_id: "graphite",
        display_name: "Graphite",
        aliases: &[],
        profile: "metrics-http",
        query_language: "graphite",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://graphite:8080",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[(
            "graphiteVersion",
            DatasourceCatalogJsonDefaultValue::String("1.1"),
        )],
    },
    DatasourceCatalogEntry {
        category: "Metrics",
        type_id: "opentsdb",
        display_name: "OpenTSDB",
        aliases: &[],
        profile: "metrics-http",
        query_language: "opentsdb",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://opentsdb:4242",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[("tsdbVersion", DatasourceCatalogJsonDefaultValue::Number(2))],
    },
    DatasourceCatalogEntry {
        category: "Logs",
        type_id: "loki",
        display_name: "Loki",
        aliases: &["grafana-loki-datasource"],
        profile: "logs-http",
        query_language: "logql",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://loki:3100",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            ("maxLines", DatasourceCatalogJsonDefaultValue::Number(1000)),
            ("timeout", DatasourceCatalogJsonDefaultValue::Number(60)),
        ],
    },
    DatasourceCatalogEntry {
        category: "Logs",
        type_id: "elasticsearch",
        display_name: "Elasticsearch",
        aliases: &[],
        profile: "logs-search-api",
        query_language: "lucene-or-query-dsl",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--user",
            "--password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://elasticsearch:9200",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: Some("@timestamp"),
        add_defaults_json_data: &[],
    },
    DatasourceCatalogEntry {
        category: "Logs",
        type_id: "opensearch",
        display_name: "OpenSearch",
        aliases: &[],
        profile: "logs-search-api",
        query_language: "ppl-or-query-dsl",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--user",
            "--password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://opensearch:9200",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: Some("@timestamp"),
        add_defaults_json_data: &[],
    },
    DatasourceCatalogEntry {
        category: "Tracing",
        type_id: "jaeger",
        display_name: "Jaeger",
        aliases: &[],
        profile: "tracing-http",
        query_language: "trace-search",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://jaeger:16686",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "nodeGraph.enabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "traceQuery.timeShiftEnabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Tracing",
        type_id: "zipkin",
        display_name: "Zipkin",
        aliases: &[],
        profile: "tracing-http",
        query_language: "trace-search",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://zipkin:9411",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "nodeGraph.enabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "traceQuery.timeShiftEnabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Tracing",
        type_id: "tempo",
        display_name: "Tempo",
        aliases: &[],
        profile: "tracing-http",
        query_language: "traceql",
        suggested_flags: &[
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ],
        target: "http-api",
        requires_url: true,
        url_example: "http://tempo:3200",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "nodeGraph.enabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "search.hide",
                DatasourceCatalogJsonDefaultValue::Bool(false),
            ),
            (
                "traceQuery.timeShiftEnabled",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "traceQuery.spanStartTimeShift",
                DatasourceCatalogJsonDefaultValue::String("-1h"),
            ),
            (
                "traceQuery.spanEndTimeShift",
                DatasourceCatalogJsonDefaultValue::String("1h"),
            ),
            (
                "streamingEnabled.search",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Databases",
        type_id: "mysql",
        display_name: "MySQL",
        aliases: &["grafana-mysql-datasource"],
        profile: "sql-database",
        query_language: "sql",
        suggested_flags: &["--user", "--password", "--tls-skip-verify", "--server-name"],
        target: "sql-database",
        requires_url: true,
        url_example: "mysql://mysql:3306",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "database",
                DatasourceCatalogJsonDefaultValue::String("grafana"),
            ),
            (
                "maxOpenConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConnsAuto",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "connMaxLifetime",
                DatasourceCatalogJsonDefaultValue::Number(14400),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Databases",
        type_id: "postgresql",
        display_name: "PostgreSQL",
        aliases: &["postgres", "grafana-postgresql-datasource"],
        profile: "sql-database",
        query_language: "sql",
        suggested_flags: &["--user", "--password", "--tls-skip-verify", "--server-name"],
        target: "sql-database",
        requires_url: true,
        url_example: "postgres://postgres:5432",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "database",
                DatasourceCatalogJsonDefaultValue::String("grafana"),
            ),
            (
                "sslmode",
                DatasourceCatalogJsonDefaultValue::String("disable"),
            ),
            (
                "maxOpenConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConnsAuto",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "connMaxLifetime",
                DatasourceCatalogJsonDefaultValue::Number(14400),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Databases",
        type_id: "mssql",
        display_name: "MSSQL",
        aliases: &[],
        profile: "sql-database",
        query_language: "sql",
        suggested_flags: &["--user", "--password", "--tls-skip-verify", "--server-name"],
        target: "sql-database",
        requires_url: true,
        url_example: "sqlserver://sqlserver:1433",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[
            (
                "database",
                DatasourceCatalogJsonDefaultValue::String("grafana"),
            ),
            (
                "maxOpenConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConns",
                DatasourceCatalogJsonDefaultValue::Number(100),
            ),
            (
                "maxIdleConnsAuto",
                DatasourceCatalogJsonDefaultValue::Bool(true),
            ),
            (
                "connMaxLifetime",
                DatasourceCatalogJsonDefaultValue::Number(14400),
            ),
            (
                "connectionTimeout",
                DatasourceCatalogJsonDefaultValue::Number(0),
            ),
        ],
    },
    DatasourceCatalogEntry {
        category: "Databases",
        type_id: "sqlite",
        display_name: "SQLite",
        aliases: &[],
        profile: "sql-database",
        query_language: "sql",
        suggested_flags: &["--user", "--password"],
        target: "embedded-database",
        requires_url: false,
        url_example: "file:/var/lib/sqlite/grafana.db",
        add_defaults_access: Some("proxy"),
        add_defaults_http_method: None,
        add_defaults_time_field: None,
        add_defaults_json_data: &[(
            "path",
            DatasourceCatalogJsonDefaultValue::String("/var/lib/sqlite/grafana.db"),
        )],
    },
];

pub fn supported_datasource_catalog() -> &'static [DatasourceCatalogEntry] {
    DATASOURCE_CATALOG
}

pub fn find_supported_datasource_entry(
    type_or_alias: &str,
) -> Option<&'static DatasourceCatalogEntry> {
    let candidate = type_or_alias.trim().to_ascii_lowercase();
    if candidate.is_empty() {
        return None;
    }
    supported_datasource_catalog().iter().find(|entry| {
        candidate == entry.type_id || entry.aliases.iter().any(|alias| candidate == *alias)
    })
}

pub fn normalize_supported_datasource_type(type_or_alias: &str) -> String {
    find_supported_datasource_entry(type_or_alias)
        .map(|entry| entry.type_id.to_string())
        .unwrap_or_else(|| type_or_alias.trim().to_string())
}

fn insert_json_data_default(json_data: &mut Map<String, Value>, key_path: &str, value: Value) {
    if let Some((prefix, suffix)) = key_path.split_once('.') {
        let child = json_data
            .entry(prefix.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(map) = child {
            insert_json_data_default(map, suffix, value);
        }
        return;
    }
    json_data.insert(key_path.to_string(), value);
}

fn build_full_json_data_scaffold(entry: &DatasourceCatalogEntry) -> Map<String, Value> {
    let mut json_data = Map::new();
    match entry.type_id {
        "loki" => {
            json_data.insert(
                "derivedFields".to_string(),
                json!([
                    {
                        "name": "TraceID",
                        "matcherRegex": "traceID=(\\w+)",
                        "datasourceUid": "tempo",
                        "url": "$${__value.raw}",
                        "urlDisplayLabel": "View Trace"
                    }
                ]),
            );
        }
        "tempo" => {
            json_data.insert(
                "serviceMap".to_string(),
                json!({
                    "datasourceUid": "prometheus"
                }),
            );
            json_data.insert(
                "tracesToLogsV2".to_string(),
                json!({
                    "datasourceUid": "loki",
                    "spanStartTimeShift": "-1h",
                    "spanEndTimeShift": "1h"
                }),
            );
            json_data.insert(
                "tracesToMetrics".to_string(),
                json!({
                    "datasourceUid": "prometheus",
                    "spanStartTimeShift": "-1h",
                    "spanEndTimeShift": "1h"
                }),
            );
        }
        "mysql" => {
            json_data.insert("tlsAuth".to_string(), Value::Bool(true));
            json_data.insert("tlsSkipVerify".to_string(), Value::Bool(true));
        }
        "postgresql" => {
            json_data.insert("postgresVersion".to_string(), Value::Number(903.into()));
            json_data.insert("timescaledb".to_string(), Value::Bool(false));
        }
        _ => {}
    }
    json_data
}

fn build_json_data_defaults(
    entry: &DatasourceCatalogEntry,
    preset_profile: DatasourcePresetProfile,
) -> Map<String, Value> {
    let mut json_data = Map::new();
    if let Some(http_method) = entry.add_defaults_http_method {
        json_data.insert(
            "httpMethod".to_string(),
            Value::String(http_method.to_string()),
        );
    }
    if let Some(time_field) = entry.add_defaults_time_field {
        json_data.insert(
            "timeField".to_string(),
            Value::String(time_field.to_string()),
        );
    }
    for (key, value) in entry.add_defaults_json_data {
        insert_json_data_default(&mut json_data, key, value.to_json_value());
    }
    if matches!(preset_profile, DatasourcePresetProfile::Full) {
        for (key, value) in build_full_json_data_scaffold(entry) {
            json_data.insert(key, value);
        }
    }
    json_data
}

fn build_add_defaults_document(entry: &DatasourceCatalogEntry) -> Value {
    let mut document = Map::new();
    if let Some(access) = entry.add_defaults_access {
        document.insert("access".to_string(), Value::String(access.to_string()));
    }
    let json_data = build_json_data_defaults(entry, DatasourcePresetProfile::Starter);
    if !json_data.is_empty() {
        document.insert("jsonData".to_string(), Value::Object(json_data));
    }
    Value::Object(document)
}

fn build_full_add_defaults_document(entry: &DatasourceCatalogEntry) -> Value {
    let mut document = Map::new();
    if let Some(access) = entry.add_defaults_access {
        document.insert("access".to_string(), Value::String(access.to_string()));
    }
    let mut json_data = build_json_data_defaults(entry, DatasourcePresetProfile::Starter);
    match entry.type_id {
        "loki" => {
            json_data.insert(
                "derivedFields".to_string(),
                json!([
                    {
                        "name": "TraceID",
                        "matcherRegex": "traceID=(\\w+)",
                        "datasourceUid": "tempo",
                        "url": "$${__value.raw}",
                        "urlDisplayLabel": "View Trace"
                    }
                ]),
            );
        }
        "tempo" => {
            json_data.insert(
                "serviceMap".to_string(),
                json!({
                    "datasourceUid": "prometheus"
                }),
            );
            json_data.insert(
                "tracesToLogsV2".to_string(),
                json!({
                    "datasourceUid": "loki",
                    "spanStartTimeShift": "-1h",
                    "spanEndTimeShift": "1h"
                }),
            );
            json_data.insert(
                "tracesToMetrics".to_string(),
                json!({
                    "datasourceUid": "prometheus",
                    "spanStartTimeShift": "-1h",
                    "spanEndTimeShift": "1h"
                }),
            );
        }
        "mysql" => {
            json_data.insert("tlsAuth".to_string(), Value::Bool(true));
            json_data.insert("tlsSkipVerify".to_string(), Value::Bool(true));
        }
        "postgresql" => {
            json_data.insert("postgresVersion".to_string(), Value::Number(903.into()));
            json_data.insert("timescaledb".to_string(), Value::Bool(false));
        }
        _ => {}
    }
    if !json_data.is_empty() {
        document.insert("jsonData".to_string(), Value::Object(json_data));
    }
    Value::Object(document)
}

fn supported_preset_profiles(entry: &DatasourceCatalogEntry) -> Vec<&'static str> {
    if build_full_add_defaults_document(entry) == build_add_defaults_document(entry) {
        vec!["starter"]
    } else {
        vec!["starter", "full"]
    }
}

pub fn build_add_defaults_for_supported_type(
    type_or_alias: &str,
    preset_profile: DatasourcePresetProfile,
) -> BTreeMap<String, Value> {
    let Some(entry) = find_supported_datasource_entry(type_or_alias) else {
        return BTreeMap::new();
    };
    let mut defaults = BTreeMap::new();
    if let Some(access) = entry.add_defaults_access {
        defaults.insert("access".to_string(), Value::String(access.to_string()));
    }
    if matches!(preset_profile, DatasourcePresetProfile::Full) {
        if let Some(http_method) = entry.add_defaults_http_method {
            defaults.insert(
                "httpMethod".to_string(),
                Value::String(http_method.to_string()),
            );
        }
        if let Some(time_field) = entry.add_defaults_time_field {
            defaults.insert(
                "timeField".to_string(),
                Value::String(time_field.to_string()),
            );
        }
    }
    let json_data = build_json_data_defaults(entry, preset_profile);
    if !json_data.is_empty() {
        defaults.insert("jsonData".to_string(), Value::Object(json_data));
    }
    defaults
}

pub fn render_supported_datasource_catalog_text() -> Vec<String> {
    let mut lines = vec!["Grafana Data Sources Summary".to_string(), String::new()];
    let mut current_category = "";
    for entry in supported_datasource_catalog() {
        if entry.category != current_category {
            if !current_category.is_empty() {
                lines.push(String::new());
            }
            current_category = entry.category;
            lines.push(format!("{}:", entry.category));
        }
        let mut line = format!("  - {} ({})", entry.display_name, entry.type_id);
        line.push_str(&format!(
            " profile={} query={}",
            entry.profile, entry.query_language
        ));
        if entry.requires_url {
            line.push_str(" url=required");
        } else {
            line.push_str(" url=optional");
        }
        let mut default_bits = Vec::new();
        if let Some(access) = entry.add_defaults_access {
            default_bits.push(format!("access={access}"));
        }
        if let Some(http_method) = entry.add_defaults_http_method {
            default_bits.push(format!("jsonData.httpMethod={http_method}"));
        }
        if let Some(time_field) = entry.add_defaults_time_field {
            default_bits.push(format!("jsonData.timeField={time_field}"));
        }
        for (key, value) in entry.add_defaults_json_data {
            default_bits.push(format!("jsonData.{key}={}", value.to_display_value()));
        }
        if !default_bits.is_empty() {
            line.push_str(&format!(" defaults: {}", default_bits.join(", ")));
        }
        if !entry.aliases.is_empty() {
            line.push_str(&format!(" aliases: {}", entry.aliases.join(", ")));
        }
        if !entry.suggested_flags.is_empty() {
            line.push_str(&format!(" flags: {}", entry.suggested_flags.join(", ")));
        }
        lines.push(line);
    }
    lines
}

pub fn render_supported_datasource_catalog_json() -> Value {
    let categories =
        supported_datasource_catalog()
            .iter()
            .fold(Vec::<Value>::new(), |mut rows, entry| {
                if let Some(last) = rows.last_mut() {
                    let same_category = last
                        .get("category")
                        .and_then(Value::as_str)
                        .map(|value| value == entry.category)
                        .unwrap_or(false);
                    if same_category {
                        last.get_mut("types")
                            .and_then(Value::as_array_mut)
                            .expect("types array")
                            .push(json!({
                                "type": entry.type_id,
                                "displayName": entry.display_name,
                                "aliases": entry.aliases,
                                "profile": entry.profile,
                                "queryLanguage": entry.query_language,
                                "requiresDatasourceUrl": entry.requires_url,
                                "suggestedFlags": entry.suggested_flags,
                                "presetProfiles": supported_preset_profiles(entry),
                                "addDefaults": build_add_defaults_document(entry),
                                "fullAddDefaults": build_full_add_defaults_document(entry),
                            }));
                        return rows;
                    }
                }
                rows.push(json!({
                        "category": entry.category,
                        "types": [{
                        "type": entry.type_id,
                        "displayName": entry.display_name,
                        "aliases": entry.aliases,
                        "profile": entry.profile,
                        "queryLanguage": entry.query_language,
                        "requiresDatasourceUrl": entry.requires_url,
                        "suggestedFlags": entry.suggested_flags,
                        "presetProfiles": supported_preset_profiles(entry),
                        "addDefaults": build_add_defaults_document(entry),
                        "fullAddDefaults": build_full_add_defaults_document(entry),
                    }],
                }));
                rows
            });
    json!({
        "kind": "grafana-utils-datasource-supported-types",
        "categories": categories,
    })
}
