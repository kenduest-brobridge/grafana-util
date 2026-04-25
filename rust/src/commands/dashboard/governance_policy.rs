//! Governance policy source resolution and file loading helpers.

use crate::common::{parse_error, validation, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use super::{GovernanceGateArgs, GovernancePolicySource};

mod yaml;

const DEFAULT_BUILTIN_POLICY_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/commands/dashboard/assets/builtin_governance_policy.json"
));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GovernancePolicyFormat {
    Json,
    Yaml,
}

pub(crate) fn built_in_governance_policy() -> Value {
    serde_json::from_str(DEFAULT_BUILTIN_POLICY_JSON)
        .expect("checked-in built-in governance policy must be valid JSON")
}

fn merge_policy_overlay(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_policy_overlay(base_value, overlay_value),
                    None => {
                        base_map.insert(key.clone(), overlay_value.clone());
                    }
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value.clone(),
    }
}

fn built_in_governance_policy_with_overlay(overlay: Value) -> Value {
    let mut policy = built_in_governance_policy();
    merge_policy_overlay(&mut policy, &overlay);
    policy
}

fn supported_builtin_policy_names() -> &'static str {
    "default, strict, balanced, lenient"
}

pub(crate) fn load_governance_policy(args: &GovernanceGateArgs) -> Result<Value> {
    load_governance_policy_source(
        args.policy_source,
        args.policy.as_deref(),
        args.builtin_policy.as_deref(),
    )
}

pub(crate) fn load_governance_policy_source(
    source: GovernancePolicySource,
    policy_path: Option<&Path>,
    builtin_policy: Option<&str>,
) -> Result<Value> {
    match source {
        GovernancePolicySource::File => {
            let policy_path = policy_path.ok_or_else(|| {
                validation(
                    "Governance gate requires --policy when --policy-source file is selected.",
                )
            })?;
            load_governance_policy_file(policy_path)
        }
        GovernancePolicySource::Builtin => {
            let policy_name = builtin_policy.unwrap_or("default");
            load_builtin_governance_policy(policy_name)
        }
    }
}

pub(crate) fn load_governance_policy_file(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    let preferred_format = preferred_policy_format(path, &raw);
    let value = parse_governance_policy_document(&raw, preferred_format).map_err(|error| {
        parse_error(
            format!("governance policy file {}", path.display()),
            format!(
                "could not be parsed as {}.\n{} parse error: {}",
                policy_format_label(preferred_format),
                policy_format_label(preferred_format),
                error
            ),
        )
    })?;
    if !value.is_object() {
        return Err(validation(format!(
            "Governance policy file {} must contain a JSON object or YAML mapping.",
            path.display()
        )));
    }
    Ok(value)
}

pub(crate) fn load_builtin_governance_policy(name: &str) -> Result<Value> {
    match name.trim().to_ascii_lowercase().as_str() {
        "default" | "example" => Ok(built_in_governance_policy()),
        "strict" => Ok(built_in_governance_policy_with_overlay(json!({
            "queries": {
                "maxQueriesPerDashboard": 40,
                "maxQueriesPerPanel": 4,
                "maxQueryComplexityScore": 4,
                "maxDashboardComplexityScore": 20
            },
            "enforcement": {
                "failOnWarnings": true
            }
        }))),
        "balanced" => Ok(built_in_governance_policy_with_overlay(json!({
            "queries": {
                "maxQueriesPerDashboard": 60,
                "maxQueriesPerPanel": 6,
                "maxQueryComplexityScore": 5,
                "maxDashboardComplexityScore": 30
            }
        }))),
        "lenient" => Ok(built_in_governance_policy_with_overlay(json!({
            "queries": {
                "maxQueriesPerDashboard": 120,
                "maxQueriesPerPanel": 12,
                "maxQueryComplexityScore": 8,
                "maxDashboardComplexityScore": 60,
                "forbidSelectStar": false,
                "requireSqlTimeFilter": false,
                "forbidBroadLokiRegex": false
            }
        }))),
        other => Err(validation(format!(
            "Unknown built-in governance policy {other:?}. Supported values: {}. Alias: example -> default.",
            supported_builtin_policy_names()
        ))),
    }
}

fn preferred_policy_format(path: &Path, raw: &str) -> GovernancePolicyFormat {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        _ if looks_like_json(raw) => GovernancePolicyFormat::Json,
        Some("json") => GovernancePolicyFormat::Json,
        Some("yaml") | Some("yml") => GovernancePolicyFormat::Yaml,
        _ => GovernancePolicyFormat::Yaml,
    }
}

fn looks_like_json(raw: &str) -> bool {
    matches!(raw.trim_start().chars().next(), Some('{') | Some('['))
}

fn policy_format_label(format: GovernancePolicyFormat) -> &'static str {
    match format {
        GovernancePolicyFormat::Json => "JSON",
        GovernancePolicyFormat::Yaml => "YAML",
    }
}

fn parse_governance_policy_document(
    raw: &str,
    format: GovernancePolicyFormat,
) -> std::result::Result<Value, String> {
    match format {
        GovernancePolicyFormat::Json => {
            serde_json::from_str(raw).map_err(|error| error.to_string())
        }
        GovernancePolicyFormat::Yaml => yaml::parse_governance_policy_yaml(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_governance_policy_source_supports_builtin_policy() {
        let policy =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, None).unwrap();

        assert_eq!(policy["version"], json!(1));
    }

    #[test]
    fn load_governance_policy_source_supports_named_builtin_policy() {
        let policy =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, Some("default"))
                .unwrap();

        assert_eq!(policy["version"], json!(1));
    }

    #[test]
    fn load_governance_policy_source_supports_strict_builtin_policy() {
        let policy =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, Some("strict"))
                .unwrap();

        assert_eq!(policy["version"], json!(1));
        assert_eq!(policy["queries"]["maxQueriesPerDashboard"], json!(40));
        assert_eq!(policy["queries"]["maxQueriesPerPanel"], json!(4));
        assert_eq!(policy["queries"]["maxQueryComplexityScore"], json!(4));
        assert_eq!(policy["queries"]["maxDashboardComplexityScore"], json!(20));
        assert_eq!(policy["enforcement"]["failOnWarnings"], json!(true));
    }

    #[test]
    fn load_governance_policy_source_supports_balanced_builtin_policy() {
        let policy =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, Some("balanced"))
                .unwrap();

        assert_eq!(policy["version"], json!(1));
        assert_eq!(policy["queries"]["maxQueriesPerDashboard"], json!(60));
        assert_eq!(policy["queries"]["maxQueriesPerPanel"], json!(6));
        assert_eq!(policy["queries"]["maxQueryComplexityScore"], json!(5));
        assert_eq!(policy["queries"]["maxDashboardComplexityScore"], json!(30));
        assert_eq!(policy["queries"]["forbidSelectStar"], json!(true));
        assert_eq!(policy["enforcement"]["failOnWarnings"], json!(false));
    }

    #[test]
    fn load_governance_policy_source_supports_lenient_builtin_policy() {
        let policy =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, Some("lenient"))
                .unwrap();

        assert_eq!(policy["version"], json!(1));
        assert_eq!(policy["queries"]["maxQueriesPerDashboard"], json!(120));
        assert_eq!(policy["queries"]["maxQueriesPerPanel"], json!(12));
        assert_eq!(policy["queries"]["maxQueryComplexityScore"], json!(8));
        assert_eq!(policy["queries"]["maxDashboardComplexityScore"], json!(60));
        assert_eq!(policy["queries"]["forbidSelectStar"], json!(false));
        assert_eq!(policy["queries"]["requireSqlTimeFilter"], json!(false));
        assert_eq!(policy["queries"]["forbidBroadLokiRegex"], json!(false));
        assert_eq!(policy["enforcement"]["failOnWarnings"], json!(false));
    }

    #[test]
    fn load_governance_policy_source_reports_supported_builtin_policy_names() {
        let error =
            load_governance_policy_source(GovernancePolicySource::Builtin, None, Some("unknown"))
                .unwrap_err()
                .to_string();

        assert!(error.contains("Supported values: default, strict, balanced, lenient."));
        assert!(error.contains("Alias: example -> default."));
    }

    #[test]
    fn load_governance_policy_file_accepts_json_policy_documents() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("policy.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "queries": {
                    "maxQueriesPerDashboard": 4
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let policy = load_governance_policy_file(&path).unwrap();

        assert_eq!(policy["version"], json!(1));
        assert_eq!(policy["queries"]["maxQueriesPerDashboard"], json!(4));
    }

    #[test]
    fn load_governance_policy_file_accepts_yaml_policy_documents() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("policy.yaml");
        fs::write(
            &path,
            r#"
version: 1
queries:
  maxQueriesPerDashboard: 2
  maxQueriesPerPanel: 1
enforcement:
  failOnWarnings: true
"#,
        )
        .unwrap();

        let policy = load_governance_policy_file(&path).unwrap();

        assert_eq!(policy["version"], json!(1));
        assert_eq!(policy["queries"]["maxQueriesPerDashboard"], json!(2));
        assert_eq!(policy["enforcement"]["failOnWarnings"], json!(true));
    }

    #[test]
    fn load_governance_policy_file_accepts_json_content_without_extension() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("policy");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "version": 1,
                "dashboards": {
                    "minRefreshIntervalSeconds": 30
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let policy = load_governance_policy_file(&path).unwrap();

        assert_eq!(policy["dashboards"]["minRefreshIntervalSeconds"], json!(30));
    }

    #[test]
    fn load_governance_policy_file_reports_yaml_parse_errors_clearly() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("policy.yaml");
        fs::write(
            &path,
            r#"
version: 1
queries:
  maxQueriesPerDashboard: 2
  maxQueriesPerPanel: 1
  invalid
"#,
        )
        .unwrap();

        let error = load_governance_policy_file(&path).unwrap_err().to_string();

        assert!(error.contains("governance policy file"));
        assert!(error.contains("YAML"));
        assert!(error.contains("line"));
    }
}
