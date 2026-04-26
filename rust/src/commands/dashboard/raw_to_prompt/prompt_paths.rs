//! Prompt-lane preservation helpers for raw-to-prompt conversions.

use serde_json::Value;
use std::collections::BTreeSet;

use super::super::RawToPromptArgs;

pub(crate) fn raw_to_prompt_live_lookup_requested(args: &RawToPromptArgs) -> bool {
    args.profile.is_some()
        || args.url.is_some()
        || args.api_token.is_some()
        || args.username.is_some()
        || args.password.is_some()
        || args.prompt_password
        || args.prompt_token
        || args.org_id.is_some()
        || args.timeout.is_some()
        || args.verify_ssl
}

pub(crate) fn is_dashboard_v2_payload(payload: &Value) -> bool {
    crate::dashboard::is_dashboard_v2_resource(payload)
}

pub(crate) fn collect_library_panel_portability_warnings(dashboard: &Value) -> Vec<String> {
    let mut count = 0usize;
    collect_library_panel_reference_count(dashboard, &mut count);
    if count == 0 {
        Vec::new()
    } else {
        vec![format!(
            "library panel external export is not fully portable yet; preserved {count} libraryPanel reference(s) without inlining models"
        )]
    }
}

fn collect_library_panel_reference_count(node: &Value, count: &mut usize) {
    match node {
        Value::Object(object) => {
            if object.contains_key("libraryPanel") {
                *count += 1;
            }
            for value in object.values() {
                collect_library_panel_reference_count(value, count);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_library_panel_reference_count(item, count);
            }
        }
        _ => {}
    }
}

pub(crate) fn collect_panel_placeholder_datasource_paths(document: &Value) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    collect_panel_placeholder_datasource_paths_recursive(document, "root", &mut paths);
    paths
}

fn collect_panel_placeholder_datasource_paths_recursive(
    node: &Value,
    current_path: &str,
    paths: &mut BTreeSet<String>,
) {
    match node {
        Value::Object(object) => {
            for (key, value) in object {
                let next_path = format!("{current_path}.{key}");
                if key == "datasource"
                    && current_path.contains(".panels[")
                    && is_placeholder_datasource_reference(value)
                {
                    paths.insert(next_path.clone());
                }
                collect_panel_placeholder_datasource_paths_recursive(value, &next_path, paths);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_panel_placeholder_datasource_paths_recursive(
                    item,
                    &format!("{current_path}[{index}]"),
                    paths,
                );
            }
        }
        _ => {}
    }
}

pub(crate) fn rewrite_prompt_panel_placeholder_paths(
    document: &mut Value,
    paths: &BTreeSet<String>,
) {
    rewrite_prompt_panel_placeholder_paths_recursive(document, "root", paths);
}

fn rewrite_prompt_panel_placeholder_paths_recursive(
    node: &mut Value,
    current_path: &str,
    paths: &BTreeSet<String>,
) {
    match node {
        Value::Object(object) => {
            if let Some(datasource) = object.get_mut("datasource") {
                let datasource_path = format!("{current_path}.datasource");
                if paths.contains(&datasource_path) {
                    *datasource = serde_json::json!({"uid": "$datasource"});
                }
            }
            for (key, value) in object {
                rewrite_prompt_panel_placeholder_paths_recursive(
                    value,
                    &format!("{current_path}.{key}"),
                    paths,
                );
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                rewrite_prompt_panel_placeholder_paths_recursive(
                    item,
                    &format!("{current_path}[{index}]"),
                    paths,
                );
            }
        }
        _ => {}
    }
}

pub(crate) fn is_placeholder_datasource_reference(reference: &Value) -> bool {
    match reference {
        Value::String(text) => text.starts_with('$'),
        Value::Object(object) => {
            object
                .get("uid")
                .and_then(Value::as_str)
                .is_some_and(|value| value.starts_with('$'))
                || object
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|value| value.starts_with('$'))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rewrite_prompt_panel_placeholder_paths_preserves_placeholder_datasource_slots() {
        let mut document = json!({
            "panels": [{
                "datasource": {"uid": "$datasource"}
            }]
        });
        let paths = collect_panel_placeholder_datasource_paths(&document);
        assert!(paths.contains("root.panels[0].datasource"));
        rewrite_prompt_panel_placeholder_paths(&mut document, &paths);
        assert_eq!(document["panels"][0]["datasource"]["uid"], "$datasource");
    }

    #[test]
    fn dashboard_v2_payload_is_detected_from_grafana_api_version() {
        let payload = json!({
            "apiVersion": "dashboard.grafana.app/v1alpha1",
            "kind": "Dashboard"
        });
        assert!(is_dashboard_v2_payload(&payload));
    }

    #[test]
    fn dashboard_v2_payload_is_detected_from_resource_spec_wrapper() {
        let payload = json!({
            "kind": "Dashboard",
            "metadata": {"name": "minimal-v2"},
            "spec": {"title": "Minimal V2"}
        });
        assert!(is_dashboard_v2_payload(&payload));
    }
}
