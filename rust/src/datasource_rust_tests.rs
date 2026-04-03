//! Datasource domain test suite.
//! Exercises parsing + import/export/diff helpers, including mocked datasource matching
//! and contract fixtures.
use super::{
    build_add_payload, build_import_payload, build_modify_payload, build_modify_updates,
    diff_datasources_with_live, discover_export_org_import_scopes, load_import_records,
    parse_json_object_argument, render_data_source_csv, render_data_source_json,
    render_data_source_table, render_import_table, render_live_mutation_json,
    render_live_mutation_table, resolve_delete_match, resolve_live_mutation_match, resolve_match,
    run_datasource_cli, CommonCliArgs, DatasourceCliArgs, DatasourceImportArgs,
    DatasourceImportRecord,
};
use crate::datasource_catalog::render_supported_datasource_catalog_json;
use clap::{CommandFactory, Parser};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

fn live_datasource(
    id: i64,
    uid: &str,
    name: &str,
    datasource_type: &str,
) -> serde_json::Map<String, Value> {
    json!({
        "id": id,
        "uid": uid,
        "name": name,
        "type": datasource_type
    })
    .as_object()
    .unwrap()
    .clone()
}

fn load_contract_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../fixtures/datasource_contract_cases.json"
    ))
    .unwrap()
}

fn load_nested_json_data_merge_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../fixtures/datasource_nested_json_data_merge_cases.json"
    ))
    .unwrap()
}

fn load_secure_json_merge_cases() -> Vec<Value> {
    serde_json::from_str(include_str!(
        "../../fixtures/datasource_secure_json_merge_cases.json"
    ))
    .unwrap()
}

fn load_preset_profile_add_payload_cases() -> Vec<Value> {
    let document: Value = serde_json::from_str(include_str!(
        "../../fixtures/datasource_preset_profile_add_payload_cases.json"
    ))
    .unwrap();
    document["cases"].as_array().cloned().unwrap()
}

fn load_supported_types_catalog_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/datasource_supported_types_catalog.json"
    ))
    .unwrap()
}

fn project_supported_types_catalog(document: &Value) -> Value {
    json!({
        "kind": document["kind"].clone(),
        "categories": document["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|category| {
                json!({
                    "category": category["category"].clone(),
                    "types": category["types"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|datasource_type| {
                            json!({
                                "type": datasource_type["type"].clone(),
                                "profile": datasource_type["profile"].clone(),
                                "queryLanguage": datasource_type["queryLanguage"].clone(),
                                "requiresDatasourceUrl": datasource_type["requiresDatasourceUrl"].clone(),
                                "suggestedFlags": datasource_type["suggestedFlags"].clone(),
                                "presetProfiles": datasource_type["presetProfiles"].clone(),
                                "addDefaults": datasource_type["addDefaults"].clone(),
                                "fullAddDefaults": datasource_type["fullAddDefaults"].clone(),
                            })
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn assert_json_subset(actual: &Value, expected: &Value) {
    match expected {
        Value::Object(expected_object) => {
            let actual_object = actual
                .as_object()
                .unwrap_or_else(|| panic!("expected object, got {actual:?}"));
            for (key, expected_value) in expected_object {
                let actual_value = actual_object
                    .get(key)
                    .unwrap_or_else(|| panic!("missing key {key} in {actual:?}"));
                assert_json_subset(actual_value, expected_value);
            }
        }
        Value::Array(expected_items) => {
            let actual_items = actual
                .as_array()
                .unwrap_or_else(|| panic!("expected array, got {actual:?}"));
            assert_eq!(actual_items.len(), expected_items.len());
            for (actual_item, expected_item) in actual_items.iter().zip(expected_items.iter()) {
                assert_json_subset(actual_item, expected_item);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

fn test_datasource_common_args() -> CommonCliArgs {
    CommonCliArgs {
        url: "http://grafana.example".to_string(),
        api_token: None,
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
    }
}

#[test]
fn datasource_root_help_includes_examples() {
    let mut command = DatasourceCliArgs::command();
    let mut output = Vec::new();
    command.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("Examples:"));
    assert!(help.contains("grafana-util datasource browse"));
    assert!(help.contains("grafana-util datasource types"));
    assert!(help.contains("grafana-util datasource list"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("grafana-util datasource add"));
    assert!(help.contains("grafana-util datasource import"));
}

#[test]
fn types_help_includes_examples() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("types")
        .unwrap_or_else(|| panic!("missing datasource types help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--json"));
    assert!(help.contains("--output-format"));
    assert!(help.contains("grafana-util datasource types"));
}

#[test]
fn list_help_explains_org_scope_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("list")
        .unwrap_or_else(|| panic!("missing datasource list help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--org-id"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("Requires Basic auth"));
    assert!(help.contains("Examples:"));
}

#[test]
fn browse_help_mentions_edit_delete_and_examples() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("browse")
        .unwrap_or_else(|| panic!("missing datasource browse help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--org-id"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("grafana-util datasource browse"));
    assert!(help.contains("edit"));
    assert!(help.contains("delete"));
}

#[test]
fn import_help_explains_common_operator_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("import")
        .unwrap_or_else(|| panic!("missing datasource import help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--import-dir"));
    assert!(help.contains("--org-id"));
    assert!(help.contains("--use-export-org"));
    assert!(help.contains("--only-org-id"));
    assert!(help.contains("--create-missing-orgs"));
    assert!(help.contains("--require-matching-export-org"));
    assert!(help.contains("--replace-existing"));
    assert!(help.contains("--update-existing-only"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("--table"));
    assert!(help.contains("--json"));
    assert!(help.contains("--output-format"));
    assert!(help.contains("--output-columns"));
    assert!(help.contains("--progress"));
    assert!(help.contains("--verbose"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("Input Options"));
}

#[test]
fn parse_datasource_browse_supports_org_scope_flag() {
    let args: DatasourceCliArgs =
        DatasourceCliArgs::parse_from(["grafana-util datasource", "browse", "--org-id", "7"]);

    match args.command {
        super::DatasourceGroupCommand::Browse(inner) => {
            assert_eq!(inner.org_id, Some(7));
            assert!(!inner.all_orgs);
        }
        _ => panic!("expected datasource browse"),
    }
}

#[test]
fn parse_datasource_browse_supports_all_orgs_flag() {
    let args: DatasourceCliArgs =
        DatasourceCliArgs::parse_from(["grafana-util datasource", "browse", "--all-orgs"]);

    match args.command {
        super::DatasourceGroupCommand::Browse(inner) => {
            assert!(inner.all_orgs);
            assert_eq!(inner.org_id, None);
        }
        _ => panic!("expected datasource browse"),
    }
}

#[test]
fn parse_datasource_browse_rejects_conflicting_org_scope_flags() {
    let result = DatasourceCliArgs::try_parse_from([
        "grafana-util datasource",
        "browse",
        "--org-id",
        "7",
        "--all-orgs",
    ]);

    assert!(result.is_err());
}

#[test]
fn export_help_explains_org_scope_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("export")
        .unwrap_or_else(|| panic!("missing datasource export help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--org-id"));
    assert!(help.contains("--all-orgs"));
    assert!(help.contains("--overwrite"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("Examples:"));
}

#[test]
fn add_help_explains_live_mutation_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("add")
        .unwrap_or_else(|| panic!("missing datasource add help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--name"));
    assert!(help.contains("--type"));
    assert!(help.contains("--apply-supported-defaults"));
    assert!(help.contains("--preset-profile"));
    assert!(help.contains("starter"));
    assert!(help.contains("full"));
    assert!(help.contains("--datasource-url"));
    assert!(help.contains("--basic-auth"));
    assert!(help.contains("--basic-auth-user"));
    assert!(help.contains("--basic-auth-password"));
    assert!(help.contains("--user"));
    assert!(help.contains("--password"));
    assert!(help.contains("--with-credentials"));
    assert!(help.contains("--http-header"));
    assert!(help.contains("--tls-skip-verify"));
    assert!(help.contains("--server-name"));
    assert!(help.contains("--json-data"));
    assert!(help.contains("--secure-json-data"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("Examples:"));
}

#[test]
fn build_add_payload_normalizes_supported_type_alias() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "grafana-prometheus-datasource",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("prometheus"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_when_requested() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--apply-supported-defaults",
        "--json-data",
        "{\"httpMethod\":\"GET\",\"timeInterval\":\"30s\"}",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["access"], json!("proxy"));
            assert!(!payload.as_object().unwrap().contains_key("httpMethod"));
            assert_eq!(payload["jsonData"]["httpMethod"], json!("GET"));
            assert_eq!(payload["jsonData"]["timeInterval"], json!("30s"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_defaults() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--preset-profile",
        "full",
        "--json-data",
        "{\"httpMethod\":\"GET\",\"timeInterval\":\"30s\"}",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["httpMethod"], json!("POST"));
            assert_eq!(payload["jsonData"]["httpMethod"], json!("GET"));
            assert_eq!(payload["jsonData"]["timeInterval"], json!("30s"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_scaffold_for_loki() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Loki Main",
        "--type",
        "loki",
        "--preset-profile",
        "full",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("loki"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["maxLines"], json!(1000));
            assert_eq!(payload["jsonData"]["timeout"], json!(60));
            assert_eq!(
                payload["jsonData"]["derivedFields"][0]["name"],
                json!("TraceID")
            );
            assert_eq!(
                payload["jsonData"]["derivedFields"][0]["matcherRegex"],
                json!("traceID=(\\w+)")
            );
            assert_eq!(
                payload["jsonData"]["derivedFields"][0]["datasourceUid"],
                json!("tempo")
            );
            assert_eq!(
                payload["jsonData"]["derivedFields"][0]["urlDisplayLabel"],
                json!("View Trace")
            );
            assert_eq!(
                payload["jsonData"]["derivedFields"][0]["url"],
                json!("$${__value.raw}")
            );
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_scaffold_for_tempo() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Tempo Main",
        "--type",
        "tempo",
        "--preset-profile",
        "full",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("tempo"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(
                payload["jsonData"]["serviceMap"]["datasourceUid"],
                json!("prometheus")
            );
            assert_eq!(
                payload["jsonData"]["tracesToLogsV2"]["datasourceUid"],
                json!("loki")
            );
            assert_eq!(
                payload["jsonData"]["tracesToLogsV2"]["spanStartTimeShift"],
                json!("-1h")
            );
            assert_eq!(
                payload["jsonData"]["tracesToLogsV2"]["spanEndTimeShift"],
                json!("1h")
            );
            assert_eq!(
                payload["jsonData"]["tracesToMetrics"]["datasourceUid"],
                json!("prometheus")
            );
            assert_eq!(
                payload["jsonData"]["tracesToMetrics"]["spanStartTimeShift"],
                json!("-1h")
            );
            assert_eq!(
                payload["jsonData"]["tracesToMetrics"]["spanEndTimeShift"],
                json!("1h")
            );
            assert_eq!(payload["jsonData"]["nodeGraph"]["enabled"], json!(true));
            assert_eq!(payload["jsonData"]["search"]["hide"], json!(false));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_scaffold_for_mysql() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "MySQL Main",
        "--type",
        "mysql",
        "--preset-profile",
        "full",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("mysql"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["database"], json!("grafana"));
            assert_eq!(payload["jsonData"]["maxOpenConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConnsAuto"], json!(true));
            assert_eq!(payload["jsonData"]["connMaxLifetime"], json!(14400));
            assert_eq!(payload["jsonData"]["tlsAuth"], json!(true));
            assert_eq!(payload["jsonData"]["tlsSkipVerify"], json!(true));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_scaffold_for_postgresql() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Postgres Main",
        "--type",
        "postgresql",
        "--preset-profile",
        "full",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("postgresql"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["database"], json!("grafana"));
            assert_eq!(payload["jsonData"]["sslmode"], json!("disable"));
            assert_eq!(payload["jsonData"]["maxOpenConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConnsAuto"], json!(true));
            assert_eq!(payload["jsonData"]["connMaxLifetime"], json!(14400));
            assert_eq!(payload["jsonData"]["postgresVersion"], json!(903));
            assert_eq!(payload["jsonData"]["timescaledb"], json!(false));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_matches_shared_preset_profile_fixture() {
    for case in load_preset_profile_add_payload_cases() {
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let add_args = match parsed.command {
            super::DatasourceGroupCommand::Add(inner) => inner,
            _ => panic!("expected datasource add"),
        };

        let payload = build_add_payload(&add_args).unwrap();
        assert_json_subset(&payload, &case["expectedSubset"]);
    }
}

#[test]
fn build_add_payload_applies_full_preset_profile_time_field_defaults() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Elastic Main",
        "--type",
        "elasticsearch",
        "--preset-profile",
        "full",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["timeField"], json!("@timestamp"));
            assert_eq!(payload["jsonData"]["timeField"], json!("@timestamp"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_for_elasticsearch() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Elastic Main",
        "--type",
        "elasticsearch",
        "--apply-supported-defaults",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("elasticsearch"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["timeField"], json!("@timestamp"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_for_influxdb() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Influx Main",
        "--type",
        "influxdb",
        "--apply-supported-defaults",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("influxdb"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["version"], json!("Flux"));
            assert_eq!(payload["jsonData"]["organization"], json!("main-org"));
            assert_eq!(payload["jsonData"]["defaultBucket"], json!("metrics"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_for_loki() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Loki Main",
        "--type",
        "loki",
        "--apply-supported-defaults",
        "--json-data",
        "{\"maxLines\":250}",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("loki"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["maxLines"], json!(250));
            assert_eq!(payload["jsonData"]["timeout"], json!(60));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_for_tempo() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Tempo Main",
        "--type",
        "tempo",
        "--apply-supported-defaults",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("tempo"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["nodeGraph"]["enabled"], json!(true));
            assert_eq!(payload["jsonData"]["search"]["hide"], json!(false));
            assert_eq!(
                payload["jsonData"]["traceQuery"]["timeShiftEnabled"],
                json!(true)
            );
            assert_eq!(
                payload["jsonData"]["traceQuery"]["spanStartTimeShift"],
                json!("-1h")
            );
            assert_eq!(
                payload["jsonData"]["traceQuery"]["spanEndTimeShift"],
                json!("1h")
            );
            assert_eq!(
                payload["jsonData"]["streamingEnabled"]["search"],
                json!(true)
            );
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn build_add_payload_applies_supported_defaults_for_postgresql() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Postgres Main",
        "--type",
        "postgres",
        "--apply-supported-defaults",
        "--dry-run",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            let payload = build_add_payload(&inner).unwrap();
            assert_eq!(payload["type"], json!("postgresql"));
            assert_eq!(payload["access"], json!("proxy"));
            assert_eq!(payload["jsonData"]["database"], json!("grafana"));
            assert_eq!(payload["jsonData"]["sslmode"], json!("disable"));
            assert_eq!(payload["jsonData"]["maxOpenConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConns"], json!(100));
            assert_eq!(payload["jsonData"]["maxIdleConnsAuto"], json!(true));
            assert_eq!(payload["jsonData"]["connMaxLifetime"], json!(14400));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn supported_catalog_json_includes_prometheus_profile_metadata() {
    let document = render_supported_datasource_catalog_json();
    let prometheus = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Metrics"))
        .and_then(|row| row["types"].as_array())
        .and_then(|rows| rows.iter().find(|row| row["type"] == json!("prometheus")))
        .unwrap();

    assert_eq!(prometheus["profile"], json!("metrics-http"));
    assert_eq!(prometheus["queryLanguage"], json!("promql"));
    assert_eq!(prometheus["requiresDatasourceUrl"], json!(true));
    assert!(prometheus["suggestedFlags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "--basic-auth"));
    assert_eq!(prometheus["presetProfiles"], json!(["starter"]));
    assert_eq!(prometheus["addDefaults"]["access"], json!("proxy"));
    assert_eq!(
        prometheus["addDefaults"]["jsonData"]["httpMethod"],
        json!("POST")
    );
    assert_eq!(prometheus["fullAddDefaults"], prometheus["addDefaults"]);
}

#[test]
fn supported_catalog_json_matches_shared_supported_types_fixture() {
    let document = render_supported_datasource_catalog_json();

    assert_eq!(
        project_supported_types_catalog(&document),
        load_supported_types_catalog_fixture()
    );
}

#[test]
fn supported_catalog_json_includes_database_profile_metadata() {
    let document = render_supported_datasource_catalog_json();
    let sqlite = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Databases"))
        .and_then(|row| row["types"].as_array())
        .and_then(|rows| rows.iter().find(|row| row["type"] == json!("sqlite")))
        .unwrap();

    assert_eq!(sqlite["profile"], json!("sql-database"));
    assert_eq!(sqlite["queryLanguage"], json!("sql"));
    assert_eq!(sqlite["requiresDatasourceUrl"], json!(false));
    assert!(sqlite["suggestedFlags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "--user"));
    assert_eq!(sqlite["presetProfiles"], json!(["starter"]));
}

#[test]
fn supported_catalog_json_includes_family_level_json_data_defaults() {
    let document = render_supported_datasource_catalog_json();
    let metrics_types = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Metrics"))
        .and_then(|row| row["types"].as_array())
        .unwrap();
    let influxdb = metrics_types
        .iter()
        .find(|row| row["type"] == json!("influxdb"))
        .unwrap();
    assert_eq!(influxdb["addDefaults"]["access"], json!("proxy"));
    assert_eq!(
        influxdb["addDefaults"]["jsonData"]["version"],
        json!("Flux")
    );
    assert_eq!(
        influxdb["addDefaults"]["jsonData"]["organization"],
        json!("main-org")
    );
    assert_eq!(
        influxdb["addDefaults"]["jsonData"]["defaultBucket"],
        json!("metrics")
    );
    let graphite = metrics_types
        .iter()
        .find(|row| row["type"] == json!("graphite"))
        .unwrap();
    assert_eq!(
        graphite["addDefaults"]["jsonData"]["graphiteVersion"],
        json!("1.1")
    );

    let logs_types = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Logs"))
        .and_then(|row| row["types"].as_array())
        .unwrap();
    let loki = logs_types
        .iter()
        .find(|row| row["type"] == json!("loki"))
        .unwrap();
    assert_eq!(loki["addDefaults"]["access"], json!("proxy"));
    assert_eq!(loki["presetProfiles"], json!(["starter", "full"]));
    assert_eq!(loki["addDefaults"]["jsonData"]["maxLines"], json!(1000));
    assert_eq!(loki["addDefaults"]["jsonData"]["timeout"], json!(60));
    assert_eq!(
        loki["fullAddDefaults"]["jsonData"]["derivedFields"][0]["datasourceUid"],
        json!("tempo")
    );

    let tracing_types = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Tracing"))
        .and_then(|row| row["types"].as_array())
        .unwrap();
    let tempo = tracing_types
        .iter()
        .find(|row| row["type"] == json!("tempo"))
        .unwrap();
    assert_eq!(tempo["presetProfiles"], json!(["starter", "full"]));
    assert_eq!(
        tempo["addDefaults"]["jsonData"]["nodeGraph"]["enabled"],
        json!(true)
    );
    assert_eq!(
        tempo["addDefaults"]["jsonData"]["search"]["hide"],
        json!(false)
    );
    assert_eq!(
        tempo["addDefaults"]["jsonData"]["traceQuery"]["timeShiftEnabled"],
        json!(true)
    );
    assert_eq!(
        tempo["fullAddDefaults"]["jsonData"]["serviceMap"]["datasourceUid"],
        json!("prometheus")
    );
    assert_eq!(
        tempo["fullAddDefaults"]["jsonData"]["tracesToLogsV2"]["datasourceUid"],
        json!("loki")
    );

    let database_types = document["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["category"] == json!("Databases"))
        .and_then(|row| row["types"].as_array())
        .unwrap();
    let postgresql = database_types
        .iter()
        .find(|row| row["type"] == json!("postgresql"))
        .unwrap();
    assert_eq!(postgresql["presetProfiles"], json!(["starter", "full"]));
    assert_eq!(
        postgresql["addDefaults"]["jsonData"]["database"],
        json!("grafana")
    );
    assert_eq!(
        postgresql["addDefaults"]["jsonData"]["sslmode"],
        json!("disable")
    );
    let mysql = database_types
        .iter()
        .find(|row| row["type"] == json!("mysql"))
        .unwrap();
    assert_eq!(mysql["presetProfiles"], json!(["starter", "full"]));
    assert_eq!(mysql["fullAddDefaults"]["jsonData"]["tlsAuth"], json!(true));
}

#[test]
fn supported_catalog_text_mentions_profile_and_flags() {
    let lines = crate::datasource_catalog::render_supported_datasource_catalog_text();
    let prometheus_line = lines
        .iter()
        .find(|line| line.contains("Prometheus (prometheus)"))
        .unwrap();
    assert!(prometheus_line.contains("profile=metrics-http"));
    assert!(prometheus_line.contains("query=promql"));
    assert!(prometheus_line.contains("flags: --basic-auth"));
}

#[test]
fn supported_catalog_text_mentions_family_level_defaults() {
    let lines = crate::datasource_catalog::render_supported_datasource_catalog_text();
    let influxdb_line = lines
        .iter()
        .find(|line| line.contains("InfluxDB (influxdb)"))
        .unwrap();
    assert!(influxdb_line.contains("defaults: access=proxy, jsonData.version=Flux"));
    assert!(influxdb_line.contains("jsonData.organization=main-org"));
    assert!(influxdb_line.contains("jsonData.defaultBucket=metrics"));

    let loki_line = lines
        .iter()
        .find(|line| line.contains("Loki (loki)"))
        .unwrap();
    assert!(loki_line.contains("jsonData.maxLines=1000"));
    assert!(loki_line.contains("jsonData.timeout=60"));

    let tempo_line = lines
        .iter()
        .find(|line| line.contains("Tempo (tempo)"))
        .unwrap();
    assert!(tempo_line.contains("jsonData.nodeGraph.enabled=true"));
    assert!(tempo_line.contains("jsonData.traceQuery.timeShiftEnabled=true"));

    let postgresql_line = lines
        .iter()
        .find(|line| line.contains("PostgreSQL (postgresql)"))
        .unwrap();
    assert!(postgresql_line.contains("jsonData.database=grafana"));
    assert!(postgresql_line.contains("jsonData.sslmode=disable"));
}

#[test]
fn delete_help_explains_live_mutation_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("delete")
        .unwrap_or_else(|| panic!("missing datasource delete help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--uid"));
    assert!(help.contains("--name"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("--yes"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("Safety Options"));
}

#[test]
fn modify_help_explains_live_mutation_flags() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("modify")
        .unwrap_or_else(|| panic!("missing datasource modify help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--uid"));
    assert!(help.contains("--set-url"));
    assert!(help.contains("--set-access"));
    assert!(help.contains("--set-default"));
    assert!(help.contains("--basic-auth"));
    assert!(help.contains("--basic-auth-user"));
    assert!(help.contains("--basic-auth-password"));
    assert!(help.contains("--user"));
    assert!(help.contains("--password"));
    assert!(help.contains("--with-credentials"));
    assert!(help.contains("--http-header"));
    assert!(help.contains("--tls-skip-verify"));
    assert!(help.contains("--server-name"));
    assert!(help.contains("--json-data"));
    assert!(help.contains("--secure-json-data"));
    assert!(help.contains("--dry-run"));
    assert!(help.contains("Examples:"));
}

#[test]
fn parse_datasource_list_supports_output_format_json() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "list",
        "--output-format",
        "json",
    ]);

    match args.command {
        super::DatasourceGroupCommand::List(inner) => {
            assert!(inner.json);
            assert!(!inner.table);
            assert!(!inner.csv);
        }
        _ => panic!("expected datasource list"),
    }
}

#[test]
fn parse_datasource_types_supports_output_format_json() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "types",
        "--output-format",
        "json",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Types(inner) => {
            assert!(inner.json);
        }
        _ => panic!("expected datasource types"),
    }
}

#[test]
fn parse_datasource_list_supports_org_scope_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "list",
        "--org-id",
        "7",
        "--output-format",
        "csv",
    ]);

    match args.command {
        super::DatasourceGroupCommand::List(inner) => {
            assert_eq!(inner.org_id, Some(7));
            assert!(!inner.all_orgs);
            assert!(inner.csv);
        }
        _ => panic!("expected datasource list"),
    }
}

#[test]
fn parse_datasource_list_supports_all_orgs_flag() {
    let args =
        DatasourceCliArgs::parse_normalized_from(["grafana-util", "list", "--all-orgs", "--json"]);

    match args.command {
        super::DatasourceGroupCommand::List(inner) => {
            assert!(inner.all_orgs);
            assert_eq!(inner.org_id, None);
            assert!(inner.json);
        }
        _ => panic!("expected datasource list"),
    }
}

#[test]
fn parse_datasource_list_rejects_conflicting_org_scope_flags() {
    let error =
        DatasourceCliArgs::try_parse_from(["grafana-util", "list", "--org-id", "7", "--all-orgs"])
            .unwrap_err();

    assert!(error.to_string().contains("--org-id"));
    assert!(error.to_string().contains("--all-orgs"));
}

#[test]
fn render_data_source_table_includes_org_columns_when_present() {
    let datasources = vec![json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "org": "Main Org.",
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let lines = render_data_source_table(&datasources, true);

    assert!(lines[0].contains("ORG"));
    assert!(lines[0].contains("ORG_ID"));
    assert!(lines[2].contains("Main Org."));
    assert!(lines[2].contains("1"));
}

#[test]
fn render_data_source_csv_and_json_include_org_fields_when_present() {
    let datasources = vec![json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "org": "Main Org.",
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let csv = render_data_source_csv(&datasources);
    let json_value = render_data_source_json(&datasources);

    assert_eq!(csv[0], "uid,name,type,url,isDefault,org,orgId");
    assert!(csv[1].contains("Main Org."));
    assert_eq!(json_value[0]["org"], Value::String("Main Org.".to_string()));
    assert_eq!(json_value[0]["orgId"], Value::String("1".to_string()));
}

#[test]
fn parse_datasource_add_supports_output_format_table() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--dry-run",
        "--output-format",
        "table",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            assert!(inner.dry_run);
            assert!(inner.table);
            assert!(!inner.json);
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn parse_datasource_add_supports_datasource_auth_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--basic-auth",
        "--basic-auth-user",
        "metrics-user",
        "--basic-auth-password",
        "metrics-pass",
        "--user",
        "query-user",
        "--password",
        "query-pass",
        "--with-credentials",
        "--http-header",
        "X-Scope-OrgID=tenant-a",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Add(inner) => {
            assert!(inner.basic_auth);
            assert_eq!(inner.basic_auth_user.as_deref(), Some("metrics-user"));
            assert_eq!(inner.basic_auth_password.as_deref(), Some("metrics-pass"));
            assert_eq!(inner.user.as_deref(), Some("query-user"));
            assert_eq!(inner.datasource_password.as_deref(), Some("query-pass"));
            assert!(inner.with_credentials);
            assert_eq!(inner.http_header, vec!["X-Scope-OrgID=tenant-a"]);
            assert!(inner.tls_skip_verify);
            assert_eq!(inner.server_name.as_deref(), Some("prometheus.internal"));
        }
        _ => panic!("expected datasource add"),
    }
}

#[test]
fn parse_datasource_delete_supports_output_format_json() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "delete",
        "--uid",
        "prom-main",
        "--dry-run",
        "--output-format",
        "json",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Delete(inner) => {
            assert_eq!(inner.uid.as_deref(), Some("prom-main"));
            assert!(inner.dry_run);
            assert!(inner.json);
            assert!(!inner.table);
        }
        _ => panic!("expected datasource delete"),
    }
}

#[test]
fn parse_datasource_delete_accepts_yes_confirmation() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "delete",
        "--uid",
        "prom-main",
        "--yes",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Delete(inner) => {
            assert_eq!(inner.uid.as_deref(), Some("prom-main"));
            assert!(inner.yes);
            assert!(!inner.dry_run);
        }
        _ => panic!("expected datasource delete"),
    }
}

#[test]
fn parse_datasource_modify_supports_output_format_table() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--set-url",
        "http://prometheus-v2:9090",
        "--dry-run",
        "--output-format",
        "table",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Modify(inner) => {
            assert_eq!(inner.uid, "prom-main");
            assert_eq!(inner.set_url.as_deref(), Some("http://prometheus-v2:9090"));
            assert!(inner.dry_run);
            assert!(inner.table);
            assert!(!inner.json);
        }
        _ => panic!("expected datasource modify"),
    }
}

#[test]
fn parse_datasource_modify_supports_datasource_auth_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--basic-auth",
        "--basic-auth-user",
        "metrics-user",
        "--basic-auth-password",
        "metrics-pass",
        "--user",
        "query-user",
        "--password",
        "query-pass",
        "--with-credentials",
        "--http-header",
        "X-Scope-OrgID=tenant-b",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Modify(inner) => {
            assert!(inner.basic_auth);
            assert_eq!(inner.basic_auth_user.as_deref(), Some("metrics-user"));
            assert_eq!(inner.basic_auth_password.as_deref(), Some("metrics-pass"));
            assert_eq!(inner.user.as_deref(), Some("query-user"));
            assert_eq!(inner.datasource_password.as_deref(), Some("query-pass"));
            assert!(inner.with_credentials);
            assert_eq!(inner.http_header, vec!["X-Scope-OrgID=tenant-b"]);
            assert!(inner.tls_skip_verify);
            assert_eq!(inner.server_name.as_deref(), Some("prometheus.internal"));
        }
        _ => panic!("expected datasource modify"),
    }
}

#[test]
fn resolve_match_marks_multiple_name_matches_as_ambiguous() {
    let record = DatasourceImportRecord {
        uid: String::new(),
        name: "Prometheus Main".to_string(),
        datasource_type: "prometheus".to_string(),
        access: "proxy".to_string(),
        url: "http://prometheus:9090".to_string(),
        is_default: true,
        org_id: "1".to_string(),
    };
    let live = vec![
        live_datasource(1, "prom-a", "Prometheus Main", "prometheus"),
        live_datasource(2, "prom-b", "Prometheus Main", "prometheus"),
    ];

    let matching = resolve_match(&record, &live, false, false);

    assert_eq!(matching.destination, "ambiguous");
    assert_eq!(matching.action, "would-fail-ambiguous");
    assert_eq!(matching.target_name, "Prometheus Main");
    assert_eq!(matching.target_id, None);
}

#[test]
fn resolve_live_mutation_match_distinguishes_uid_name_mismatch() {
    let live = vec![live_datasource(
        7,
        "prom-main",
        "Prometheus Main",
        "prometheus",
    )];

    let matching = resolve_live_mutation_match(Some("prom-main"), Some("Other Name"), &live);

    assert_eq!(matching.destination, "uid-name-mismatch");
    assert_eq!(matching.action, "would-fail-uid-name-mismatch");
    assert_eq!(matching.target_id, Some(7));
}

#[test]
fn resolve_delete_match_returns_would_delete_for_existing_uid() {
    let live = vec![live_datasource(
        7,
        "prom-main",
        "Prometheus Main",
        "prometheus",
    )];

    let matching = resolve_delete_match(Some("prom-main"), None, &live);

    assert_eq!(matching.destination, "exists-uid");
    assert_eq!(matching.action, "would-delete");
    assert_eq!(matching.target_id, Some(7));
}

#[test]
fn resolve_match_allows_update_when_uid_exists_and_replace_existing_is_enabled() {
    let record = DatasourceImportRecord {
        uid: "prom-main".to_string(),
        name: "Prometheus Main".to_string(),
        datasource_type: "prometheus".to_string(),
        access: "proxy".to_string(),
        url: "http://prometheus:9090".to_string(),
        is_default: true,
        org_id: "1".to_string(),
    };
    let live = vec![live_datasource(
        9,
        "prom-main",
        "Prometheus Main",
        "prometheus",
    )];

    let matching = resolve_match(&record, &live, true, false);

    assert_eq!(matching.destination, "exists-uid");
    assert_eq!(matching.action, "would-update");
    assert_eq!(matching.target_uid, "prom-main");
    assert_eq!(matching.target_id, Some(9));
}

#[test]
fn resolve_match_blocks_name_match_when_uid_differs() {
    let record = DatasourceImportRecord {
        uid: "prom-export".to_string(),
        name: "Prometheus Main".to_string(),
        datasource_type: "prometheus".to_string(),
        access: "proxy".to_string(),
        url: "http://prometheus:9090".to_string(),
        is_default: true,
        org_id: "1".to_string(),
    };
    let live = vec![live_datasource(
        9,
        "prom-live",
        "Prometheus Main",
        "prometheus",
    )];

    let matching = resolve_match(&record, &live, true, false);

    assert_eq!(matching.destination, "exists-name");
    assert_eq!(matching.action, "would-fail-uid-mismatch");
    assert_eq!(matching.target_uid, "prom-live");
    assert_eq!(matching.target_id, Some(9));
}

#[test]
fn render_import_table_can_omit_header() {
    let rows = vec![vec![
        "prom-main".to_string(),
        "Prometheus Main".to_string(),
        "prometheus".to_string(),
        "exists-uid".to_string(),
        "would-update".to_string(),
        "7".to_string(),
        "datasources.json#0".to_string(),
    ]];

    let lines = render_import_table(&rows, false, None);

    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("prom-main"));
    assert!(!lines[0].contains("UID"));
}

#[test]
fn render_import_table_honors_selected_columns() {
    let rows = vec![vec![
        "prom-main".to_string(),
        "Prometheus Main".to_string(),
        "prometheus".to_string(),
        "exists-uid".to_string(),
        "would-update".to_string(),
        "7".to_string(),
        "datasources.json#0".to_string(),
    ]];

    let lines = render_import_table(
        &rows,
        true,
        Some(&[
            "uid".to_string(),
            "action".to_string(),
            "org_id".to_string(),
        ]),
    );

    assert!(lines[0].contains("UID"));
    assert!(lines[0].contains("ACTION"));
    assert!(lines[0].contains("ORG_ID"));
    assert!(!lines[0].contains("NAME"));
    assert!(lines[2].contains("prom-main"));
    assert!(lines[2].contains("would-update"));
    assert!(lines[2].contains("7"));
}

#[test]
fn parse_datasource_import_preserves_requested_path() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./datasources",
        "--org-id",
        "7",
        "--dry-run",
        "--table",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Import(inner) => {
            assert_eq!(inner.import_dir, Path::new("./datasources"));
            assert_eq!(inner.org_id, Some(7));
            assert!(inner.dry_run);
            assert!(inner.table);
        }
        _ => panic!("expected datasource import"),
    }
}

#[test]
fn parse_datasource_import_supports_output_format_table() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./datasources",
        "--dry-run",
        "--output-format",
        "table",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Import(inner) => {
            assert!(inner.dry_run);
            assert!(inner.table);
            assert!(!inner.json);
        }
        _ => panic!("expected datasource import"),
    }
}

#[test]
fn parse_datasource_import_supports_output_columns() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./datasources",
        "--dry-run",
        "--output-format",
        "table",
        "--output-columns",
        "uid,action,orgId,file",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Import(inner) => {
            assert!(inner.table);
            assert_eq!(
                inner.output_columns,
                vec!["uid", "action", "org_id", "file"]
            );
        }
        _ => panic!("expected datasource import"),
    }
}

#[test]
fn parse_datasource_export_supports_org_scope_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "export",
        "--export-dir",
        "./datasources",
        "--org-id",
        "7",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Export(inner) => {
            assert_eq!(inner.export_dir, Path::new("./datasources"));
            assert_eq!(inner.org_id, Some(7));
            assert!(!inner.all_orgs);
        }
        _ => panic!("expected datasource export"),
    }
}

#[test]
fn parse_datasource_export_supports_all_orgs_flag() {
    let args = DatasourceCliArgs::parse_normalized_from(["grafana-util", "export", "--all-orgs"]);

    match args.command {
        super::DatasourceGroupCommand::Export(inner) => {
            assert!(inner.all_orgs);
            assert_eq!(inner.org_id, None);
        }
        _ => panic!("expected datasource export"),
    }
}

#[test]
fn parse_datasource_import_supports_use_export_org_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./datasources",
        "--use-export-org",
        "--only-org-id",
        "2",
        "--only-org-id",
        "5",
        "--create-missing-orgs",
    ]);

    match args.command {
        super::DatasourceGroupCommand::Import(inner) => {
            assert!(inner.use_export_org);
            assert_eq!(inner.only_org_id, vec![2, 5]);
            assert!(inner.create_missing_orgs);
            assert_eq!(inner.org_id, None);
        }
        _ => panic!("expected datasource import"),
    }
}

#[test]
fn parse_datasource_import_rejects_org_id_with_use_export_org() {
    let error = DatasourceCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--import-dir",
        "./datasources",
        "--org-id",
        "7",
        "--use-export-org",
    ])
    .unwrap_err();

    assert!(error.to_string().contains("--org-id"));
    assert!(error.to_string().contains("--use-export-org"));
}

#[test]
fn build_import_payload_matches_shared_contract_fixtures() {
    for case in load_contract_cases() {
        let object = case.as_object().unwrap();
        let normalized = object
            .get("expectedNormalizedRecord")
            .and_then(Value::as_object)
            .unwrap();
        let expected_payload = object.get("expectedImportPayload").cloned().unwrap();
        let record = DatasourceImportRecord {
            uid: normalized
                .get("uid")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            name: normalized
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            datasource_type: normalized
                .get("type")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            access: normalized
                .get("access")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            url: normalized
                .get("url")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            is_default: normalized.get("isDefault").and_then(Value::as_str).unwrap() == "true",
            org_id: normalized
                .get("orgId")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
        };

        assert_eq!(build_import_payload(&record), expected_payload);
    }
}

#[test]
fn build_add_payload_keeps_optional_json_fields() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--uid",
        "prom-main",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--access",
        "proxy",
        "--datasource-url",
        "http://prometheus:9090",
        "--default",
        "--json-data",
        r#"{"httpMethod":"POST"}"#,
        "--secure-json-data",
        r#"{"httpHeaderValue1":"secret"}"#,
    ]);
    let add_args = match args.command {
        super::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let payload = build_add_payload(&add_args).unwrap();

    assert_eq!(
        payload,
        json!({
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "access": "proxy",
            "url": "http://prometheus:9090",
            "isDefault": true,
            "jsonData": {"httpMethod": "POST"},
            "secureJsonData": {"httpHeaderValue1": "secret"}
        })
    );
}

#[test]
fn build_add_payload_supports_datasource_auth_and_header_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--uid",
        "prom-main",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--datasource-url",
        "http://prometheus:9090",
        "--apply-supported-defaults",
        "--basic-auth",
        "--basic-auth-user",
        "metrics-user",
        "--basic-auth-password",
        "metrics-pass",
        "--user",
        "query-user",
        "--password",
        "query-pass",
        "--with-credentials",
        "--http-header",
        "X-Scope-OrgID=tenant-a",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);
    let add_args = match args.command {
        super::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let payload = build_add_payload(&add_args).unwrap();

    assert_eq!(payload["basicAuth"], json!(true));
    assert_eq!(payload["basicAuthUser"], json!("metrics-user"));
    assert_eq!(payload["user"], json!("query-user"));
    assert_eq!(payload["withCredentials"], json!(true));
    assert_eq!(payload["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(
        payload["jsonData"]["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(payload["jsonData"]["tlsSkipVerify"], json!(true));
    assert_eq!(
        payload["jsonData"]["serverName"],
        json!("prometheus.internal")
    );
    assert_eq!(
        payload["secureJsonData"]["basicAuthPassword"],
        json!("metrics-pass")
    );
    assert_eq!(payload["secureJsonData"]["password"], json!("query-pass"));
    assert_eq!(
        payload["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-a")
    );
}

#[test]
fn build_add_payload_rejects_basic_auth_password_without_user() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--basic-auth-password",
        "metrics-pass",
    ]);
    let add_args = match args.command {
        super::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let error = build_add_payload(&add_args).unwrap_err().to_string();
    assert!(error.contains("requires --basic-auth-user"));
}

#[test]
fn build_add_payload_merges_nested_json_data_override_from_shared_fixture() {
    for case in load_nested_json_data_merge_cases() {
        if case["operation"] != json!("add") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let add_args = match parsed.command {
            super::DatasourceGroupCommand::Add(inner) => inner,
            _ => panic!("expected datasource add"),
        };

        let payload = build_add_payload(&add_args).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn build_modify_updates_keeps_optional_json_fields() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--set-url",
        "http://prometheus-v2:9090",
        "--set-access",
        "direct",
        "--set-default",
        "true",
        "--json-data",
        r#"{"httpMethod":"POST"}"#,
        "--secure-json-data",
        r#"{"token":"abc123"}"#,
    ]);
    let modify_args = match args.command {
        super::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };

    let updates = build_modify_updates(&modify_args).unwrap();

    assert_eq!(updates["url"], json!("http://prometheus-v2:9090"));
    assert_eq!(updates["access"], json!("direct"));
    assert_eq!(updates["isDefault"], json!(true));
    assert_eq!(updates["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(updates["secureJsonData"]["token"], json!("abc123"));
}

#[test]
fn build_modify_updates_supports_datasource_auth_and_header_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--basic-auth",
        "--basic-auth-user",
        "metrics-user",
        "--basic-auth-password",
        "metrics-pass",
        "--user",
        "query-user",
        "--password",
        "query-pass",
        "--with-credentials",
        "--http-header",
        "X-Scope-OrgID=tenant-b",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);
    let modify_args = match args.command {
        super::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };

    let updates = build_modify_updates(&modify_args).unwrap();

    assert_eq!(updates["basicAuth"], json!(true));
    assert_eq!(updates["basicAuthUser"], json!("metrics-user"));
    assert_eq!(updates["user"], json!("query-user"));
    assert_eq!(updates["withCredentials"], json!(true));
    assert_eq!(
        updates["jsonData"]["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(updates["jsonData"]["tlsSkipVerify"], json!(true));
    assert_eq!(
        updates["jsonData"]["serverName"],
        json!("prometheus.internal")
    );
    assert_eq!(
        updates["secureJsonData"]["basicAuthPassword"],
        json!("metrics-pass")
    );
    assert_eq!(updates["secureJsonData"]["password"], json!("query-pass"));
    assert_eq!(
        updates["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-b")
    );
}

#[test]
fn build_modify_payload_merges_existing_json_data() {
    let existing = json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "access": "proxy",
        "isDefault": false,
        "basicAuth": true,
        "basicAuthUser": "metrics-user",
        "user": "query-user",
        "withCredentials": true,
        "jsonData": {
            "httpMethod": "POST"
        }
    })
    .as_object()
    .unwrap()
    .clone();
    let updates = json!({
        "url": "http://prometheus-v2:9090",
        "jsonData": {
            "timeInterval": "30s"
        },
        "secureJsonData": {
            "token": "abc123"
        }
    })
    .as_object()
    .unwrap()
    .clone();

    let payload = build_modify_payload(&existing, &updates).unwrap();

    assert_eq!(payload["url"], json!("http://prometheus-v2:9090"));
    assert_eq!(payload["basicAuth"], json!(true));
    assert_eq!(payload["basicAuthUser"], json!("metrics-user"));
    assert_eq!(payload["user"], json!("query-user"));
    assert_eq!(payload["withCredentials"], json!(true));
    assert_eq!(payload["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(payload["jsonData"]["timeInterval"], json!("30s"));
    assert_eq!(payload["secureJsonData"]["token"], json!("abc123"));
}

#[test]
fn build_modify_payload_rejects_basic_auth_password_without_basic_auth_user() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--basic-auth-password",
        "metrics-pass",
    ]);
    let modify_args = match args.command {
        super::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };
    let updates = build_modify_updates(&modify_args).unwrap();
    let existing = json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "access": "proxy",
        "isDefault": false
    })
    .as_object()
    .unwrap()
    .clone();

    let error = build_modify_payload(&existing, &updates).unwrap_err();

    assert!(error
        .to_string()
        .contains("--basic-auth-password requires --basic-auth-user or an existing basicAuthUser"));
}

#[test]
fn build_modify_payload_deep_merges_nested_json_data_override_from_shared_fixture() {
    for case in load_nested_json_data_merge_cases() {
        if case["operation"] != json!("modify") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let modify_args = match parsed.command {
            super::DatasourceGroupCommand::Modify(inner) => inner,
            _ => panic!("expected datasource modify"),
        };

        let updates = build_modify_updates(&modify_args).unwrap();
        let existing = case["existing"].as_object().unwrap().clone();
        let payload = build_modify_payload(&existing, &updates).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn build_add_payload_preserves_explicit_secure_json_data_from_shared_fixture() {
    for case in load_secure_json_merge_cases() {
        if case["operation"] != json!("add") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let add_args = match parsed.command {
            super::DatasourceGroupCommand::Add(inner) => inner,
            _ => panic!("expected datasource add"),
        };

        let payload = build_add_payload(&add_args).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn build_modify_payload_replaces_secure_json_data_from_shared_fixture() {
    for case in load_secure_json_merge_cases() {
        if case["operation"] != json!("modify") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let modify_args = match parsed.command {
            super::DatasourceGroupCommand::Modify(inner) => inner,
            _ => panic!("expected datasource modify"),
        };

        let updates = build_modify_updates(&modify_args).unwrap();
        let existing = case["existing"].as_object().unwrap().clone();
        let payload = build_modify_payload(&existing, &updates).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn parse_json_object_argument_rejects_non_object_values() {
    let error = parse_json_object_argument(Some("[]"), "--json-data").unwrap_err();

    assert!(error
        .to_string()
        .contains("--json-data must decode to a JSON object."));
}

#[test]
fn render_live_mutation_table_can_omit_header() {
    let rows = vec![vec![
        "add".to_string(),
        "prom-main".to_string(),
        "Prometheus Main".to_string(),
        "prometheus".to_string(),
        "missing".to_string(),
        "would-create".to_string(),
        String::new(),
    ]];

    let lines = render_live_mutation_table(&rows, false);

    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("would-create"));
    assert!(!lines[0].contains("OPERATION"));
}

#[test]
fn render_live_mutation_json_summarizes_actions() {
    let value = render_live_mutation_json(&[
        vec![
            "add".to_string(),
            "prom-main".to_string(),
            "Prometheus Main".to_string(),
            "prometheus".to_string(),
            "missing".to_string(),
            "would-create".to_string(),
            String::new(),
        ],
        vec![
            "modify".to_string(),
            "prom-mid".to_string(),
            "Prometheus Updated".to_string(),
            "prometheus".to_string(),
            "exists-uid".to_string(),
            "would-update".to_string(),
            "9".to_string(),
        ],
        vec![
            "delete".to_string(),
            "prom-main".to_string(),
            "Prometheus Main".to_string(),
            String::new(),
            "exists-uid".to_string(),
            "would-delete".to_string(),
            "7".to_string(),
        ],
        vec![
            "add".to_string(),
            String::new(),
            "Prometheus Main".to_string(),
            "prometheus".to_string(),
            "exists-name".to_string(),
            "would-fail-existing-name".to_string(),
            "7".to_string(),
        ],
    ]);

    assert_eq!(value["summary"]["itemCount"], json!(4));
    assert_eq!(value["summary"]["createCount"], json!(1));
    assert_eq!(value["summary"]["updateCount"], json!(1));
    assert_eq!(value["summary"]["deleteCount"], json!(1));
    assert_eq!(value["summary"]["blockedCount"], json!(1));
}

#[test]
fn routed_datasource_scope_identity_matches_table_json_and_progress_surfaces() {
    let dry_run = json!({
        "mode": "create-or-update",
        "orgs": [
            {
                "sourceOrgId": 2,
                "sourceOrgName": "Org Two",
                "orgAction": "exists",
                "targetOrgId": 2,
                "datasourceCount": 1,
                "importDir": "/tmp/datasource-export-all-orgs/org_2_Org_Two"
            },
            {
                "sourceOrgId": 9,
                "sourceOrgName": "Ops Org",
                "orgAction": "would-create",
                "targetOrgId": Value::Null,
                "datasourceCount": 1,
                "importDir": "/tmp/datasource-export-all-orgs/org_9_Ops_Org"
            }
        ],
        "imports": [
            {
                "sourceOrgId": 2,
                "sourceOrgName": "Org Two",
                "orgAction": "exists",
                "targetOrgId": 2
            },
            {
                "sourceOrgId": 9,
                "sourceOrgName": "Ops Org",
                "orgAction": "would-create",
                "targetOrgId": Value::Null
            }
        ]
    });
    let org_entries = dry_run["orgs"].as_array().unwrap();
    let org_two = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let org_nine = org_entries
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    let rows: Vec<Vec<String>> = org_entries
        .iter()
        .map(|entry| {
            vec![
                entry["sourceOrgId"].as_i64().unwrap().to_string(),
                entry["sourceOrgName"].as_str().unwrap().to_string(),
                entry["orgAction"].as_str().unwrap().to_string(),
                super::format_routed_datasource_target_org_label(entry["targetOrgId"].as_i64()),
                entry["datasourceCount"].as_u64().unwrap().to_string(),
                entry["importDir"].as_str().unwrap().to_string(),
            ]
        })
        .collect();
    let table_lines = super::render_routed_datasource_import_org_table(&rows, true);

    let existing_summary = super::format_routed_datasource_scope_summary_fields(
        2,
        "Org Two",
        "exists",
        Some(2),
        Path::new(org_two["importDir"].as_str().unwrap()),
    );
    let would_create_summary = super::format_routed_datasource_scope_summary_fields(
        9,
        "Ops Org",
        "would-create",
        None,
        Path::new(org_nine["importDir"].as_str().unwrap()),
    );

    assert_eq!(org_two["targetOrgId"], json!(2));
    assert_eq!(org_nine["targetOrgId"], Value::Null);
    assert!(table_lines[2].contains("Org Two"));
    assert!(table_lines[2].contains("2"));
    assert!(table_lines[3].contains("Ops Org"));
    assert!(table_lines[3].contains("<new>"));
    assert!(existing_summary.contains("orgAction=exists"));
    assert!(existing_summary.contains("targetOrgId=2"));
    assert!(would_create_summary.contains("orgAction=would-create"));
    assert!(would_create_summary.contains("targetOrgId=<new>"));
}

#[test]
fn routed_datasource_status_matrix_covers_exists_missing_would_create_and_created() {
    let missing_payload = json!({
        "summary": {
            "existingOrgCount": 1,
            "missingOrgCount": 1,
            "wouldCreateOrgCount": 0
        },
        "orgs": [
            {
                "sourceOrgId": 2,
                "sourceOrgName": "Org Two",
                "orgAction": "exists",
                "targetOrgId": 2,
                "importDir": "/tmp/datasource-export-all-orgs/org_2_Org_Two"
            },
            {
                "sourceOrgId": 9,
                "sourceOrgName": "Ops Org",
                "orgAction": "missing",
                "targetOrgId": Value::Null,
                "importDir": "/tmp/datasource-export-all-orgs/org_9_Ops_Org"
            }
        ]
    });
    let would_create_payload = json!({
        "summary": {
            "existingOrgCount": 1,
            "missingOrgCount": 0,
            "wouldCreateOrgCount": 1
        },
        "orgs": [
            {
                "sourceOrgId": 2,
                "sourceOrgName": "Org Two",
                "orgAction": "exists",
                "targetOrgId": 2,
                "importDir": "/tmp/datasource-export-all-orgs/org_2_Org_Two"
            },
            {
                "sourceOrgId": 9,
                "sourceOrgName": "Ops Org",
                "orgAction": "would-create",
                "targetOrgId": Value::Null,
                "importDir": "/tmp/datasource-export-all-orgs/org_9_Ops_Org"
            }
        ]
    });

    let missing_orgs = missing_payload["orgs"].as_array().unwrap();
    let would_create_orgs = would_create_payload["orgs"].as_array().unwrap();
    let missing_existing = missing_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let missing_missing = missing_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();
    let would_create_existing = would_create_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(2))
        .unwrap();
    let would_create_missing = would_create_orgs
        .iter()
        .find(|entry| entry["sourceOrgId"] == json!(9))
        .unwrap();

    assert_eq!(missing_payload["summary"]["existingOrgCount"], json!(1));
    assert_eq!(missing_payload["summary"]["missingOrgCount"], json!(1));
    assert_eq!(missing_payload["summary"]["wouldCreateOrgCount"], json!(0));
    assert_eq!(
        would_create_payload["summary"]["existingOrgCount"],
        json!(1)
    );
    assert_eq!(would_create_payload["summary"]["missingOrgCount"], json!(0));
    assert_eq!(
        would_create_payload["summary"]["wouldCreateOrgCount"],
        json!(1)
    );

    assert_eq!(missing_existing["orgAction"], json!("exists"));
    assert_eq!(missing_existing["targetOrgId"], json!(2));
    assert_eq!(missing_missing["orgAction"], json!("missing"));
    assert_eq!(missing_missing["targetOrgId"], Value::Null);
    assert_eq!(would_create_existing["orgAction"], json!("exists"));
    assert_eq!(would_create_existing["targetOrgId"], json!(2));
    assert_eq!(would_create_missing["orgAction"], json!("would-create"));
    assert_eq!(would_create_missing["targetOrgId"], Value::Null);

    let existing_summary = super::format_routed_datasource_scope_summary_fields(
        2,
        "Org Two",
        "exists",
        Some(2),
        Path::new(missing_existing["importDir"].as_str().unwrap()),
    );
    let missing_summary = super::format_routed_datasource_scope_summary_fields(
        9,
        "Ops Org",
        "missing",
        None,
        Path::new(missing_missing["importDir"].as_str().unwrap()),
    );
    let would_create_summary = super::format_routed_datasource_scope_summary_fields(
        9,
        "Ops Org",
        "would-create",
        None,
        Path::new(would_create_missing["importDir"].as_str().unwrap()),
    );
    let created_summary = super::format_routed_datasource_scope_summary_fields(
        9,
        "Ops Org",
        "created",
        Some(19),
        Path::new(would_create_missing["importDir"].as_str().unwrap()),
    );
    assert!(existing_summary.contains("orgAction=exists"));
    assert!(existing_summary.contains("targetOrgId=2"));
    assert!(missing_summary.contains("orgAction=missing"));
    assert!(missing_summary.contains("targetOrgId=<new>"));
    assert!(would_create_summary.contains("orgAction=would-create"));
    assert!(would_create_summary.contains("targetOrgId=<new>"));
    assert!(created_summary.contains("orgAction=created"));
    assert!(created_summary.contains("targetOrgId=19"));
}

#[test]
fn datasource_import_rejects_output_columns_without_table_output() {
    let temp = tempdir().unwrap();
    let import_dir = temp.path().join("datasources");
    fs::create_dir_all(&import_dir).unwrap();
    fs::write(
        import_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([])).unwrap(),
    )
    .unwrap();

    let error = run_datasource_cli(
        DatasourceCliArgs::parse_normalized_from([
            "grafana-util",
            "import",
            "--import-dir",
            import_dir.to_str().unwrap(),
            "--token",
            "token",
            "--dry-run",
            "--output-columns",
            "uid",
        ])
        .command,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("--output-columns is only supported with --dry-run --table"));
}

#[test]
fn datasource_import_rejects_extra_secret_or_server_managed_fields() {
    let temp = tempdir().unwrap();
    let import_dir = temp.path().join("datasources");
    fs::create_dir_all(&import_dir).unwrap();
    fs::write(
        import_dir.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "root",
            "resource": "datasource",
            "datasourcesFile": "datasources.json",
            "indexFile": "index.json",
            "datasourceCount": 1,
            "format": "grafana-datasource-inventory-v1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        import_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([{
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "access": "proxy",
            "url": "http://prometheus:9090",
            "isDefault": true,
            "org": "Main Org.",
            "orgId": "1",
            "id": 7,
            "secureJsonData": {"httpHeaderValue1": "secret"}
        }]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        import_dir.join("index.json"),
        serde_json::to_string_pretty(&json!({"items": []})).unwrap(),
    )
    .unwrap();

    let error = load_import_records(&import_dir).unwrap_err();

    assert!(error
        .to_string()
        .contains("unsupported datasource field(s): id, secureJsonData"));
}

#[test]
fn discover_export_org_import_scopes_reads_selected_multi_org_root() {
    let temp = tempdir().unwrap();
    let import_root = write_multi_org_import_fixture(
        temp.path(),
        &[
            (
                1,
                "Main Org",
                vec![
                    json!({"uid":"prom-main","name":"Prometheus Main","type":"prometheus","access":"proxy","url":"http://prometheus:9090","isDefault":"true","org":"Main Org","orgId":"1"}),
                ],
            ),
            (
                2,
                "Org Two",
                vec![
                    json!({"uid":"prom-two","name":"Prometheus Two","type":"prometheus","access":"proxy","url":"http://prometheus-2:9090","isDefault":"false","org":"Org Two","orgId":"2"}),
                ],
            ),
        ],
    );
    let args = DatasourceImportArgs {
        common: test_datasource_common_args(),
        import_dir: import_root,
        org_id: None,
        use_export_org: true,
        only_org_id: vec![2],
        create_missing_orgs: false,
        require_matching_export_org: false,
        replace_existing: false,
        update_existing_only: false,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let scopes = discover_export_org_import_scopes(&args).unwrap();

    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0].source_org_id, 2);
    assert_eq!(scopes[0].source_org_name, "Org Two");
}

#[test]
fn discover_export_org_import_scopes_errors_when_selected_org_missing() {
    let temp = tempdir().unwrap();
    let import_root = write_multi_org_import_fixture(
        temp.path(),
        &[(
            1,
            "Main Org",
            vec![
                json!({"uid":"prom-main","name":"Prometheus Main","type":"prometheus","access":"proxy","url":"http://prometheus:9090","isDefault":"true","org":"Main Org","orgId":"1"}),
            ],
        )],
    );
    let args = DatasourceImportArgs {
        common: test_datasource_common_args(),
        import_dir: import_root,
        org_id: None,
        use_export_org: true,
        only_org_id: vec![9],
        create_missing_orgs: false,
        require_matching_export_org: false,
        replace_existing: false,
        update_existing_only: false,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        progress: false,
        verbose: false,
    };

    let error = discover_export_org_import_scopes(&args).unwrap_err();

    assert!(error
        .to_string()
        .contains("Selected exported org IDs were not found"));
}

#[test]
fn datasource_import_with_use_export_org_requires_basic_auth() {
    let temp = tempdir().unwrap();
    let import_root = write_multi_org_import_fixture(
        temp.path(),
        &[(
            1,
            "Main Org",
            vec![
                json!({"uid":"prom-main","name":"Prometheus Main","type":"prometheus","access":"proxy","url":"http://prometheus:9090","isDefault":"true","org":"Main Org","orgId":"1"}),
            ],
        )],
    );

    let error = run_datasource_cli(
        DatasourceCliArgs::parse_normalized_from([
            "grafana-util",
            "import",
            "--url",
            "http://grafana.example",
            "--token",
            "token",
            "--import-dir",
            import_root.to_str().unwrap(),
            "--use-export-org",
            "--dry-run",
        ])
        .command,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Datasource import with --use-export-org requires Basic auth"));
}

#[test]
fn diff_help_explains_diff_dir_flag() {
    let mut command = DatasourceCliArgs::command();
    let subcommand = command
        .find_subcommand_mut("diff")
        .unwrap_or_else(|| panic!("missing datasource diff help"));
    let mut output = Vec::new();
    subcommand.write_long_help(&mut output).unwrap();
    let help = String::from_utf8(output).unwrap();

    assert!(help.contains("--diff-dir"));
    assert!(help.contains("Compare datasource inventory"));
}

#[test]
fn parse_datasource_diff_preserves_requested_path() {
    let args =
        DatasourceCliArgs::parse_from(["grafana-util", "diff", "--diff-dir", "./datasources"]);

    match args.command {
        super::DatasourceGroupCommand::Diff(inner) => {
            assert_eq!(inner.diff_dir, Path::new("./datasources"));
        }
        _ => panic!("expected datasource diff"),
    }
}

#[test]
fn diff_datasources_with_live_returns_zero_for_matching_inventory() {
    let diff_dir = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let (compared_count, differences) = diff_datasources_with_live(&diff_dir, &live).unwrap();

    assert_eq!(compared_count, 1);
    assert_eq!(differences, 0);
    fs::remove_dir_all(diff_dir).unwrap();
}

#[test]
fn diff_datasources_with_live_detects_changed_inventory() {
    let diff_dir = write_diff_fixture(&[json!({
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "proxy",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })]);
    let live = vec![json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "access": "direct",
        "url": "http://prometheus:9090",
        "isDefault": true,
        "orgId": "1"
    })
    .as_object()
    .unwrap()
    .clone()];

    let (compared_count, differences) = diff_datasources_with_live(&diff_dir, &live).unwrap();

    assert_eq!(compared_count, 1);
    assert_eq!(differences, 1);
    fs::remove_dir_all(diff_dir).unwrap();
}

fn write_diff_fixture(records: &[Value]) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("grafana-util-datasource-diff-{unique}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("export-metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "root",
            "resource": "datasource",
            "datasourcesFile": "datasources.json",
            "indexFile": "index.json",
            "datasourceCount": records.len(),
            "format": "grafana-datasource-inventory-v1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("datasources.json"),
        serde_json::to_vec_pretty(&Value::Array(records.to_vec())).unwrap(),
    )
    .unwrap();
    fs::write(
        dir.join("index.json"),
        serde_json::to_vec_pretty(&json!({"items": []})).unwrap(),
    )
    .unwrap();
    dir
}

fn write_multi_org_import_fixture(
    root: &Path,
    orgs: &[(i64, &str, Vec<Value>)],
) -> std::path::PathBuf {
    let import_root = root.join("datasource-export-all-orgs");
    fs::create_dir_all(&import_root).unwrap();
    for (org_id, org_name, records) in orgs {
        let org_dir = import_root.join(format!("org_{}_{}", org_id, org_name.replace(' ', "_")));
        fs::create_dir_all(&org_dir).unwrap();
        fs::write(
            org_dir.join("export-metadata.json"),
            serde_json::to_vec_pretty(&json!({
                "schemaVersion": 1,
                "kind": "grafana-utils-datasource-export-index",
                "variant": "root",
                "resource": "datasource",
                "datasourcesFile": "datasources.json",
                "indexFile": "index.json",
                "datasourceCount": records.len(),
                "format": "grafana-datasource-inventory-v1"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            org_dir.join("datasources.json"),
            serde_json::to_vec_pretty(&Value::Array(records.clone())).unwrap(),
        )
        .unwrap();
        let index_items = records
            .iter()
            .map(|record| {
                let object = record.as_object().unwrap();
                json!({
                    "uid": object.get("uid").cloned().unwrap_or(Value::String(String::new())),
                    "name": object.get("name").cloned().unwrap_or(Value::String(String::new())),
                    "type": object.get("type").cloned().unwrap_or(Value::String(String::new())),
                    "org": object.get("org").cloned().unwrap_or(Value::String(org_name.to_string())),
                    "orgId": object.get("orgId").cloned().unwrap_or(Value::String(org_id.to_string())),
                })
            })
            .collect::<Vec<Value>>();
        fs::write(
            org_dir.join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "kind": "grafana-utils-datasource-export-index",
                "schemaVersion": 1,
                "datasourcesFile": "datasources.json",
                "count": records.len(),
                "items": index_items
            }))
            .unwrap(),
        )
        .unwrap();
    }
    import_root
}
