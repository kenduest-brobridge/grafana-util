use crate::common::{message, sanitize_path_component, Result};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

use super::{
    build_policies_export_document, detect_document_kind, discover_alert_resource_files,
    load_alert_resource_file, value_to_string, AlertCliArgs, AlertResourceKind,
    CONTACT_POINTS_SUBDIR, CONTACT_POINT_KIND, MUTE_TIMING_KIND, POLICIES_KIND, POLICIES_SUBDIR,
    RULES_SUBDIR, RULE_KIND, TEMPLATE_KIND,
};

pub(crate) fn resource_kind_to_document_kind(kind: AlertResourceKind) -> &'static str {
    match kind {
        AlertResourceKind::Rule => RULE_KIND,
        AlertResourceKind::ContactPoint => CONTACT_POINT_KIND,
        AlertResourceKind::MuteTiming => MUTE_TIMING_KIND,
        AlertResourceKind::PolicyTree => POLICIES_KIND,
        AlertResourceKind::Template => TEMPLATE_KIND,
    }
}

pub(crate) fn scaffold_output_path(
    desired_dir: &Path,
    subdir: &str,
    name: &str,
    file_suffix: &str,
) -> PathBuf {
    desired_dir
        .join(subdir)
        .join(format!("{}{file_suffix}", sanitize_path_component(name)))
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(crate) fn require_desired_dir<'a>(args: &'a AlertCliArgs, label: &str) -> Result<&'a Path> {
    args.desired_dir
        .as_deref()
        .ok_or_else(|| message(format!("{label} requires --desired-dir.")))
}

pub(crate) fn require_scaffold_name<'a>(args: &'a AlertCliArgs, label: &str) -> Result<&'a str> {
    args.scaffold_name
        .as_deref()
        .ok_or_else(|| message(format!("{label} requires --name.")))
}

pub(crate) fn require_source_name<'a>(args: &'a AlertCliArgs, label: &str) -> Result<&'a str> {
    args.source_name
        .as_deref()
        .ok_or_else(|| message(format!("{label} requires --source.")))
}

pub(crate) fn parse_string_pairs(values: &[String], label: &str) -> Result<Map<String, Value>> {
    let mut pairs = Map::new();
    for entry in values {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(message(format!(
                "{label} entries must use key=value form: {entry}"
            )));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(message(format!("{label} key cannot be empty: {entry}")));
        }
        pairs.insert(key.to_string(), Value::String(value.trim().to_string()));
    }
    Ok(pairs)
}

pub(crate) fn rule_output_path_for_name(desired_dir: &Path, name: &str) -> PathBuf {
    scaffold_output_path(desired_dir, RULES_SUBDIR, name, ".yaml")
}

pub(crate) fn contact_point_output_path_for_name(desired_dir: &Path, name: &str) -> PathBuf {
    scaffold_output_path(desired_dir, CONTACT_POINTS_SUBDIR, name, ".yaml")
}

fn preferred_policy_path(desired_dir: &Path) -> PathBuf {
    desired_dir
        .join(POLICIES_SUBDIR)
        .join("notification-policies.yaml")
}

fn resolve_policy_path(desired_dir: &Path) -> PathBuf {
    let candidates = [
        desired_dir
            .join(POLICIES_SUBDIR)
            .join("notification-policies.yaml"),
        desired_dir
            .join(POLICIES_SUBDIR)
            .join("notification-policies.yml"),
        desired_dir
            .join(POLICIES_SUBDIR)
            .join("notification-policies.json"),
    ];
    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| preferred_policy_path(desired_dir))
}

fn build_default_policy_document(receiver: &str) -> Value {
    let policy = json!({
        "receiver": if receiver.trim().is_empty() { "grafana-default-email" } else { receiver.trim() },
        "group_by": ["grafana_folder", "alertname"],
        "routes": [],
    });
    let mut document = build_policies_export_document(
        policy
            .as_object()
            .expect("default notification policies must be an object"),
    );
    if let Some(metadata) = document.get_mut("metadata").and_then(Value::as_object_mut) {
        metadata.insert(
            "managedBy".to_string(),
            Value::String("grafana-utils".to_string()),
        );
    }
    document
}

pub(crate) fn load_or_init_policy_document(
    desired_dir: &Path,
    receiver: &str,
) -> Result<(PathBuf, Value)> {
    let path = resolve_policy_path(desired_dir);
    if path.exists() {
        Ok((
            path.clone(),
            load_alert_resource_file(&path, "Notification policies")?,
        ))
    } else {
        Ok((path, build_default_policy_document(receiver)))
    }
}

pub(crate) fn extract_policy_spec(document: &Value) -> Result<Value> {
    if let Some(spec) = document.get("spec") {
        if spec.is_object() {
            return Ok(spec.clone());
        }
    }
    if document.is_object() {
        return Ok(document.clone());
    }
    Err(message(
        "Notification policies document must be a tool document or object.",
    ))
}

pub(crate) fn build_desired_route_document(args: &AlertCliArgs) -> Result<Option<Value>> {
    if args.no_route {
        return Ok(None);
    }
    let Some(receiver) = args.receiver.as_deref() else {
        return Ok(None);
    };
    let labels = parse_string_pairs(&args.labels, "Alert route label")?;
    let mut matchers = labels
        .into_iter()
        .map(|(key, value)| json!([key, "=", value_to_string(&value)]))
        .collect::<Vec<Value>>();
    if let Some(severity) = args.severity.as_deref() {
        matchers.push(json!(["severity", "=", severity]));
    }
    Ok(Some(json!({
        "receiver": receiver,
        "group_by": ["grafana_folder", "alertname"],
        "continue": false,
        "object_matchers": matchers,
    })))
}

pub(crate) fn find_existing_rule_document(
    desired_dir: &Path,
    source: &str,
) -> Result<(PathBuf, Value)> {
    let normalized_source = sanitize_path_component(source);
    for path in discover_alert_resource_files(desired_dir)? {
        let document = load_alert_resource_file(&path, "Alert source rule")?;
        let kind = document
            .as_object()
            .and_then(|item| detect_document_kind(item).ok());
        if kind != Some(RULE_KIND) {
            continue;
        }
        let metadata = document.get("metadata").and_then(Value::as_object);
        let spec = document.get("spec").and_then(Value::as_object);
        let candidates = [
            metadata
                .and_then(|item| item.get("uid"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
            metadata
                .and_then(|item| item.get("title"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
            spec.and_then(|item| item.get("uid"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
            spec.and_then(|item| item.get("title"))
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ];
        if candidates.iter().any(|candidate| {
            !candidate.is_empty()
                && (candidate == &source || sanitize_path_component(candidate) == normalized_source)
        }) {
            return Ok((path, document));
        }
    }
    Err(message(format!(
        "Could not find staged alert rule source {:?} under {}.",
        source,
        desired_dir.display()
    )))
}
