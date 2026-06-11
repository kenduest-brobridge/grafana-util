#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use serde_json::{Map, Value};

use crate::common::{message, string_field, Result};
use crate::dashboard::{build_auth_context, build_http_client_for_org, DEFAULT_ORG_ID};
use crate::http::JsonHttpClient;
use crate::interactive_browser::{browser_detail_fact, browser_detail_fallback_fact};

use super::datasource_browse_support_review::review_lines_from_datasource_details;
use super::DatasourceBrowseArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DatasourceBrowseItemKind {
    Org,
    Datasource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasourceBrowseItem {
    pub(crate) kind: DatasourceBrowseItemKind,
    pub(crate) depth: u16,
    pub(crate) id: i64,
    pub(crate) uid: String,
    pub(crate) name: String,
    pub(crate) datasource_type: String,
    pub(crate) access: String,
    pub(crate) url: String,
    pub(crate) is_default: bool,
    pub(crate) org: String,
    pub(crate) org_id: String,
    pub(crate) details: Map<String, Value>,
    pub(crate) datasource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DatasourceBrowseDocument {
    pub(crate) scope_label: String,
    pub(crate) org: String,
    pub(crate) org_id: String,
    pub(crate) items: Vec<DatasourceBrowseItem>,
    pub(crate) org_count: usize,
    pub(crate) datasource_count: usize,
}

impl DatasourceBrowseItem {
    pub(crate) fn is_org_row(&self) -> bool {
        self.kind == DatasourceBrowseItemKind::Org
    }
}

pub(crate) fn load_datasource_browse_document(
    client: &JsonHttpClient,
    args: &DatasourceBrowseArgs,
) -> Result<DatasourceBrowseDocument> {
    if args.all_orgs {
        return load_all_orgs_document(&args.common, client);
    }
    load_single_org_document(client)
}

pub(crate) fn detail_lines(item: &DatasourceBrowseItem) -> Vec<String> {
    if item.is_org_row() {
        return vec![
            browser_detail_fallback_fact("Org", &item.org, "-"),
            browser_detail_fallback_fact("Org ID", &item.org_id, "-"),
            browser_detail_fact("Datasources", item.datasource_count),
        ];
    }

    let mut lines = vec![
        browser_detail_fact("ID", item.id),
        browser_detail_fallback_fact("UID", &item.uid, "-"),
        browser_detail_fallback_fact("Name", &item.name, "-"),
        browser_detail_fallback_fact("Type", &item.datasource_type, "-"),
        browser_detail_fallback_fact("URL", &item.url, "-"),
        browser_detail_fallback_fact("Access", &item.access, "-"),
        browser_detail_fact("Default", if item.is_default { "true" } else { "false" }),
        browser_detail_fallback_fact("Org", &item.org, "-"),
        browser_detail_fallback_fact("Org ID", &item.org_id, "-"),
    ];

    if let Some(user) = item.details.get("user").and_then(Value::as_str) {
        if !user.trim().is_empty() {
            lines.push(browser_detail_fallback_fact("User", user, "-"));
        }
    }
    if let Some(value) = item.details.get("basicAuth").and_then(Value::as_bool) {
        lines.push(browser_detail_fact("Basic auth", value));
    }
    if let Some(value) = item.details.get("withCredentials").and_then(Value::as_bool) {
        lines.push(browser_detail_fact("With credentials", value));
    }
    if let Some(database) = item.details.get("database").and_then(Value::as_str) {
        if !database.trim().is_empty() {
            lines.push(browser_detail_fallback_fact("Database", database, "-"));
        }
    }
    if let Some(json_data) = item.details.get("jsonData").and_then(Value::as_object) {
        if !json_data.is_empty() {
            let keys = sorted_object_keys(json_data).join(", ");
            lines.push(browser_detail_fact("jsonData keys", keys));
        }
    }
    if let Some(secure_json_fields) = item
        .details
        .get("secureJsonFields")
        .and_then(Value::as_object)
    {
        if !secure_json_fields.is_empty() {
            let keys = sorted_object_keys(secure_json_fields).join(", ");
            lines.push(browser_detail_fact("secureJsonFields", keys));
        }
    }
    lines
}

pub(crate) fn review_lines(item: &DatasourceBrowseItem) -> Vec<String> {
    if item.is_org_row() {
        return Vec::new();
    }
    review_lines_from_datasource_details(&item.details)
}

pub(crate) fn datasource_browser_detail_lines_from_details(
    details: &Map<String, Value>,
) -> Vec<String> {
    let name = string_field(details, "name", "");
    let uid = string_field(details, "uid", "");
    let datasource_type = string_field(details, "type", "");
    let org = string_field(details, "org", "");
    let org_id = string_field(details, "orgId", "");
    let url = string_field(details, "url", "");
    let access = string_field(details, "access", "");
    let is_default = bool_or_string_field(details, "isDefault");

    vec![
        browser_detail_fallback_fact("Name", &name, "-"),
        browser_detail_fallback_fact("UID", &uid, "-"),
        browser_detail_fallback_fact("Type", &datasource_type, "-"),
        browser_detail_fact(
            "Org",
            format!("{} ({})", blank_dash(&org), blank_dash(&org_id)),
        ),
        browser_detail_fallback_fact("URL", &url, "-"),
        browser_detail_fallback_fact("Access", &access, "-"),
        browser_detail_fallback_fact("Default", &is_default, "-"),
    ]
}

fn bool_or_string_field(details: &Map<String, Value>, name: &str) -> String {
    if let Some(value) = details.get(name).and_then(Value::as_bool) {
        return value.to_string();
    }
    string_field(details, name, "")
}

fn blank_dash(value: &str) -> &str {
    if value.trim().is_empty() {
        "-"
    } else {
        value.trim()
    }
}

fn sorted_object_keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

pub(crate) fn build_modify_updates_from_browse(
    item: &DatasourceBrowseItem,
    name: &str,
    url: &str,
    access: &str,
    is_default: bool,
) -> Map<String, Value> {
    let mut updates = Map::new();
    if name.trim() != item.name {
        updates.insert("name".to_string(), Value::String(name.trim().to_string()));
    }
    if url.trim() != item.url {
        updates.insert("url".to_string(), Value::String(url.trim().to_string()));
    }
    if access.trim() != item.access {
        updates.insert(
            "access".to_string(),
            Value::String(access.trim().to_string()),
        );
    }
    if is_default != item.is_default {
        updates.insert("isDefault".to_string(), Value::Bool(is_default));
    }
    updates
}

pub(crate) fn fetch_datasource_by_uid(
    client: &JsonHttpClient,
    uid: &str,
) -> Result<Map<String, Value>> {
    super::fetch_datasource_by_uid_if_exists(client, uid)?.ok_or_else(|| {
        message(format!(
            "Datasource browse could not find live datasource UID {uid}."
        ))
    })
}

fn load_single_org_document(client: &JsonHttpClient) -> Result<DatasourceBrowseDocument> {
    let org = super::fetch_current_org(client)?;
    let org_name = string_field(&org, "name", "");
    let org_id = org
        .get("id")
        .map(|value| value.to_string())
        .unwrap_or_else(|| DEFAULT_ORG_ID.to_string());
    let items = datasource_rows_for_org(client, &org_name, &org_id, 0)?;
    let datasource_count = items.len();
    Ok(DatasourceBrowseDocument {
        scope_label: format!(
            "Org {} (id={})",
            display_value(&org_name, "-"),
            display_value(&org_id, "-")
        ),
        org: org_name,
        org_id,
        items,
        org_count: 1,
        datasource_count,
    })
}

fn load_all_orgs_document(
    common: &super::CommonCliArgs,
    client: &JsonHttpClient,
) -> Result<DatasourceBrowseDocument> {
    let context = build_auth_context(common)?;
    if context.auth_mode != "basic" {
        return Err(message(
            "Datasource browse with --all-orgs requires Basic auth (--basic-user / --basic-password).",
        ));
    }

    let mut orgs = super::list_orgs(client)?;
    orgs.sort_by(|left, right| {
        string_field(left, "name", "")
            .to_ascii_lowercase()
            .cmp(&string_field(right, "name", "").to_ascii_lowercase())
            .then_with(|| {
                left.get("id")
                    .map(Value::to_string)
                    .cmp(&right.get("id").map(Value::to_string))
            })
    });

    let mut items = Vec::new();
    let mut datasource_count = 0usize;
    for org in &orgs {
        let org_name = string_field(org, "name", "");
        let org_id = org.get("id").and_then(Value::as_i64).unwrap_or(1);
        let org_id_text = org_id.to_string();
        let scoped_client = build_http_client_for_org(common, org_id)?;
        let datasource_items = datasource_rows_for_org(&scoped_client, &org_name, &org_id_text, 1)?;
        datasource_count += datasource_items.len();
        items.push(org_row(
            org_name,
            org_id_text,
            datasource_items.len(),
            org.clone(),
        ));
        items.extend(datasource_items);
    }

    Ok(DatasourceBrowseDocument {
        scope_label: "All visible orgs".to_string(),
        org: "All visible orgs".to_string(),
        org_id: "-".to_string(),
        items,
        org_count: orgs.len(),
        datasource_count,
    })
}

fn datasource_rows_for_org(
    client: &JsonHttpClient,
    org_name: &str,
    org_id: &str,
    depth: u16,
) -> Result<Vec<DatasourceBrowseItem>> {
    let mut items = super::build_list_records(client)?
        .into_iter()
        .map(|record| datasource_row(record, org_name, org_id, depth))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            })
            .then_with(|| {
                left.uid
                    .to_ascii_lowercase()
                    .cmp(&right.uid.to_ascii_lowercase())
            })
    });
    Ok(items)
}

fn datasource_row(
    record: Map<String, Value>,
    org_name: &str,
    org_id: &str,
    depth: u16,
) -> DatasourceBrowseItem {
    DatasourceBrowseItem {
        kind: DatasourceBrowseItemKind::Datasource,
        depth,
        id: record.get("id").and_then(Value::as_i64).unwrap_or_default(),
        uid: string_field(&record, "uid", ""),
        name: string_field(&record, "name", ""),
        datasource_type: string_field(&record, "type", ""),
        access: string_field(&record, "access", ""),
        url: string_field(&record, "url", ""),
        is_default: record
            .get("isDefault")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        org: string_field(&record, "org", org_name),
        org_id: string_field(&record, "orgId", org_id),
        details: record,
        datasource_count: 0,
    }
}

fn org_row(
    org_name: String,
    org_id: String,
    datasource_count: usize,
    details: Map<String, Value>,
) -> DatasourceBrowseItem {
    DatasourceBrowseItem {
        kind: DatasourceBrowseItemKind::Org,
        depth: 0,
        id: 0,
        uid: String::new(),
        name: org_name.clone(),
        datasource_type: "org".to_string(),
        access: String::new(),
        url: String::new(),
        is_default: false,
        org: org_name,
        org_id,
        details,
        datasource_count,
    }
}

fn display_value<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn datasource_item(details: Map<String, Value>) -> DatasourceBrowseItem {
        DatasourceBrowseItem {
            kind: DatasourceBrowseItemKind::Datasource,
            depth: 0,
            id: 12,
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus".to_string(),
            is_default: true,
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            details,
            datasource_count: 0,
        }
    }

    #[test]
    fn review_lines_render_live_secure_json_fields_as_review_required_placeholders() {
        let item = datasource_item(
            json!({
                "secureJsonFields": {
                    "httpHeaderValue1": true,
                    "basicAuthPassword": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);
        let facts = detail_lines(&item);

        assert!(lines.contains(
            &"Secret placeholders: unavailable from live secureJsonFields (2 field(s): basicAuthPassword, httpHeaderValue1)".to_string()
        ));
        assert!(lines.contains(
            &"Secret blocker status: review-required; resolved values are hidden by Grafana"
                .to_string()
        ));
        assert!(lines.contains(&"Secret review required: true (secure fields present)".to_string()));
        assert!(!facts
            .iter()
            .any(|line| line.starts_with("Secret review required:")));
    }

    #[test]
    fn detail_lines_sort_json_data_and_secure_json_field_keys() {
        let item = datasource_item(
            json!({
                "jsonData": {
                    "zulu": true,
                    "alpha": true
                },
                "secureJsonFields": {
                    "httpHeaderValue1": true,
                    "basicAuthPassword": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = detail_lines(&item);

        assert!(lines.contains(&"jsonData keys: alpha, zulu".to_string()));
        assert!(
            lines.contains(&"secureJsonFields: basicAuthPassword, httpHeaderValue1".to_string())
        );
    }

    #[test]
    fn detail_lines_use_shared_browser_fact_formatting() {
        let mut item = datasource_item(
            json!({
                "user": " prom-user ",
                "basicAuth": true,
                "withCredentials": false,
                "database": " prometheus ",
                "jsonData": {
                    "zulu": true,
                    "alpha": true
                },
                "secureJsonFields": {
                    "basicAuthPassword": true
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        item.uid = "  ".to_string();

        assert_eq!(
            detail_lines(&item),
            vec![
                "ID: 12",
                "UID: -",
                "Name: Prometheus Main",
                "Type: prometheus",
                "URL: http://prometheus",
                "Access: proxy",
                "Default: true",
                "Org: Main Org.",
                "Org ID: 1",
                "User: prom-user",
                "Basic auth: true",
                "With credentials: false",
                "Database: prometheus",
                "jsonData keys: alpha, zulu",
                "secureJsonFields: basicAuthPassword",
            ]
        );
    }

    #[test]
    fn datasource_browser_detail_lines_from_details_formats_local_artifact_identity() {
        let details = json!({
            "name": "Prometheus",
            "uid": "prom-main",
            "type": "prometheus",
            "org": "Main Org.",
            "orgId": "1",
            "url": "http://prometheus:9090",
            "access": "proxy",
            "isDefault": true
        })
        .as_object()
        .unwrap()
        .clone();

        assert_eq!(
            datasource_browser_detail_lines_from_details(&details),
            vec![
                "Name: Prometheus".to_string(),
                "UID: prom-main".to_string(),
                "Type: prometheus".to_string(),
                "Org: Main Org. (1)".to_string(),
                "URL: http://prometheus:9090".to_string(),
                "Access: proxy".to_string(),
                "Default: true".to_string(),
            ]
        );
    }

    #[test]
    fn review_lines_render_placeholder_backed_secret_review_without_raw_tokens() {
        let item = datasource_item(
            json!({
                "secureJsonDataPlaceholders": {
                    "httpHeaderValue1": "${secret:prom-header}",
                    "basicAuthPassword": "${secret:prom-basic-auth}"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);
        let facts = detail_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(
            &"Secret placeholders: available (2 field(s): basicAuthPassword, httpHeaderValue1)"
                .to_string()
        ));
        assert!(
            lines.contains(&"Secret placeholder names: prom-basic-auth, prom-header".to_string())
        );
        assert!(lines.contains(
            &"Secret blocker status: blocked until --secret-values resolves placeholders"
                .to_string()
        ));
        assert!(lines.contains(
            &"Secret review required: true (placeholder-backed secureJsonData)".to_string()
        ));
        assert!(!rendered.contains("${secret:"));
        assert!(!facts.iter().any(|line| line.contains("${secret:")));
        assert!(!facts
            .iter()
            .any(|line| line.starts_with("Secret blocker status:")));
    }

    #[test]
    fn review_lines_do_not_display_resolved_secure_json_data_values() {
        let item = datasource_item(
            json!({
                "secureJsonData": {
                    "password": "super-secret-value"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);
        let facts = detail_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(
            &"Secret material: hidden (1 secureJsonData field(s): password)".to_string()
        ));
        assert!(lines.contains(
            &"Secret review required: true (resolved secureJsonData hidden)".to_string()
        ));
        assert!(!rendered.contains("super-secret-value"));
        assert!(!facts.iter().any(|line| line.contains("super-secret-value")));
    }

    #[test]
    fn review_lines_surface_provider_references_without_raw_tokens() {
        let item = datasource_item(
            json!({
                "secureJsonDataProviders": {
                    "httpHeaderValue1": "${provider:aws-sm:prod/prom/header}",
                    "basicAuthPassword": "${provider:vault:secret/data/prom/basic-auth}"
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(
            &"Secret providers: external (2 field(s): basicAuthPassword, httpHeaderValue1)"
                .to_string()
        ));
        assert!(lines.contains(&"Secret provider names: vault, aws-sm".to_string()));
        assert!(lines.contains(
            &"Secret review required: true (provider-backed secureJsonData)".to_string()
        ));
        assert!(!rendered.contains("${provider:"));
        assert!(!rendered.contains("secret/data/prom/basic-auth"));
        assert!(!rendered.contains("prod/prom/header"));
    }

    #[test]
    fn review_lines_surface_read_only_datasource_as_blocked_evidence() {
        let item = datasource_item(
            json!({
                "readOnly": true
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);

        assert!(lines.contains(
            &"Datasource blocker status: blocked; read-only datasource requires external update"
                .to_string()
        ));
        assert!(
            lines.contains(&"Datasource review required: true (read-only datasource)".to_string())
        );
    }

    #[test]
    fn review_lines_surface_plan_action_evidence_from_details() {
        let details = json!({
                "action": "blocked-read-only",
                "status": "blocked",
                "matchBasis": "uid",
                "destination": "exists-uid",
                "blockedReason": "target-read-only",
                "file": "datasources.json#0",
                "targetUid": "prom-live",
                "targetVersion": 12,
                "targetReadOnly": true,
                "changedFields": ["url", "jsonData"],
                "reviewHints": ["requires-secret-values"]
        })
        .as_object()
        .unwrap()
        .clone();
        let item = datasource_item(details.clone());

        let lines = review_lines(&item);
        let detail_lines = review_lines_from_datasource_details(&details);

        assert!(lines.contains(&"Review action: blocked-read-only (status=blocked)".to_string()));
        assert!(lines.contains(&"Review blocker status: blocked by target-read-only".to_string()));
        assert!(lines.contains(&"Review match: uid".to_string()));
        assert!(lines.contains(&"Review destination: exists-uid".to_string()));
        assert!(lines.contains(&"Review source: datasources.json#0".to_string()));
        assert!(lines.contains(&"Review target UID: prom-live".to_string()));
        assert!(lines.contains(&"Review target version: 12".to_string()));
        assert!(lines.contains(&"Review target: read-only=true".to_string()));
        assert!(lines.contains(&"Review changed fields: jsonData, url".to_string()));
        assert!(lines.contains(&"Review hints: requires-secret-values".to_string()));
        assert_eq!(detail_lines, lines);
    }

    #[test]
    fn review_lines_surface_import_dry_run_review_required_flag() {
        let item = datasource_item(
            json!({
                "action": "resolve-provider-secrets",
                "reviewRequired": true,
                "requiresSecretValues": true
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);

        assert!(lines.contains(&"Review action: resolve-provider-secrets".to_string()));
        assert!(lines.contains(&"Review required: true".to_string()));
        assert!(lines.contains(&"Review requires secret values: true".to_string()));
    }

    #[test]
    fn review_lines_surface_diff_status_without_secret_change_paths() {
        let item = datasource_item(
            json!({
                "status": "different",
                "matchBasis": "uid",
                "changedFields": ["url", "secureJsonData.password"]
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let lines = review_lines(&item);
        let rendered = lines.join("\n");

        assert!(lines.contains(&"Review status: different".to_string()));
        assert!(lines.contains(&"Review match: uid".to_string()));
        assert!(lines.contains(&"Review changed fields: url".to_string()));
        assert!(!rendered.contains("secureJsonData.password"));
    }
}
