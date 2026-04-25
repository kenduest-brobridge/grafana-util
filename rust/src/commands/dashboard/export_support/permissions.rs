use serde_json::{Map, Value};

const PERMISSION_BUNDLE_KIND: &str = "grafana-utils-dashboard-permission-bundle";
const PERMISSION_BUNDLE_SCHEMA_VERSION: i64 = 1;
const PERMISSION_EXPORT_KIND: &str = "grafana-utils-dashboard-permission-export";
const PERMISSION_EXPORT_SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PermissionExportTarget {
    pub(crate) resource_kind: &'static str,
    pub(crate) resource_uid: String,
    pub(crate) resource_title: String,
    pub(crate) org_name: String,
    pub(crate) org_id: String,
}

fn normalize_permission_level(record: &Map<String, Value>) -> (i64, &'static str) {
    let level_value = match record.get("permission") {
        Some(Value::Number(number)) => number.as_i64().unwrap_or(0),
        Some(Value::String(text)) => match text.trim().to_lowercase().as_str() {
            "view" => 1,
            "edit" => 2,
            "admin" => 4,
            other => other.parse::<i64>().unwrap_or(0),
        },
        Some(value) => value.to_string().parse::<i64>().unwrap_or(0),
        None => match record.get("permissionName") {
            Some(Value::String(text)) => match text.trim().to_lowercase().as_str() {
                "view" => 1,
                "edit" => 2,
                "admin" => 4,
                other => other.parse::<i64>().unwrap_or(0),
            },
            Some(value) => value.to_string().parse::<i64>().unwrap_or(0),
            None => 0,
        },
    };
    let level_name = match level_value {
        1 => "view",
        2 => "edit",
        4 => "admin",
        _ => "unknown",
    };
    (level_value, level_name)
}

pub(crate) fn value_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string().trim_matches('"').to_string(),
    }
}

fn normalize_permission_subject(record: &Map<String, Value>) -> (String, String, String, String) {
    let user_id = value_text(record.get("userId"));
    if !user_id.is_empty() {
        let user_name = value_text(record.get("user").or_else(|| record.get("userLogin")));
        let subject_name = if user_name.is_empty() {
            user_id.clone()
        } else {
            user_name
        };
        return (
            "user".to_string(),
            format!("user:{user_id}"),
            user_id,
            subject_name,
        );
    }
    let team_id = value_text(record.get("teamId"));
    if !team_id.is_empty() {
        let team_name = value_text(record.get("team").or_else(|| record.get("teamName")));
        let subject_name = if team_name.is_empty() {
            team_id.clone()
        } else {
            team_name
        };
        return (
            "team".to_string(),
            format!("team:{team_id}"),
            team_id,
            subject_name,
        );
    }
    let service_account_id = value_text(record.get("serviceAccountId"));
    if !service_account_id.is_empty() {
        let service_account_name = value_text(
            record
                .get("serviceAccount")
                .or_else(|| record.get("serviceAccountName")),
        );
        let subject_name = if service_account_name.is_empty() {
            service_account_id.clone()
        } else {
            service_account_name
        };
        return (
            "service-account".to_string(),
            format!("service-account:{service_account_id}"),
            service_account_id,
            subject_name,
        );
    }
    let role = value_text(record.get("role"));
    if !role.is_empty() {
        return (
            "role".to_string(),
            format!("role:{role}"),
            role.clone(),
            role,
        );
    }
    (
        "unknown".to_string(),
        "unknown".to_string(),
        String::new(),
        "unknown".to_string(),
    )
}

pub(crate) fn build_permission_export_document(
    resource_kind: &str,
    resource_uid: &str,
    resource_title: &str,
    permissions: &[Map<String, Value>],
    org_name: &str,
    org_id: &str,
) -> Value {
    let mut rows = permissions
        .iter()
        .map(|record| {
            let (subject_type, subject_key, subject_id, subject_name) =
                normalize_permission_subject(record);
            let (permission, permission_name) = normalize_permission_level(record);
            let inherited = record
                .get("inherited")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut row = Map::new();
            row.insert(
                "resourceKind".to_string(),
                Value::String(resource_kind.to_string()),
            );
            row.insert(
                "resourceUid".to_string(),
                Value::String(resource_uid.to_string()),
            );
            row.insert(
                "resourceTitle".to_string(),
                Value::String(resource_title.to_string()),
            );
            row.insert("subjectType".to_string(), Value::String(subject_type));
            row.insert("subjectKey".to_string(), Value::String(subject_key));
            row.insert("subjectId".to_string(), Value::String(subject_id));
            row.insert("subjectName".to_string(), Value::String(subject_name));
            row.insert("permission".to_string(), Value::from(permission));
            row.insert(
                "permissionName".to_string(),
                Value::String(permission_name.to_string()),
            );
            row.insert("inherited".to_string(), Value::Bool(inherited));
            row
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (
            value_text(left.get("resourceKind")),
            value_text(left.get("resourceUid")),
            value_text(left.get("subjectType")),
            value_text(left.get("subjectName")),
            left.get("permission")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        )
            .cmp(&(
                value_text(right.get("resourceKind")),
                value_text(right.get("resourceUid")),
                value_text(right.get("subjectType")),
                value_text(right.get("subjectName")),
                right
                    .get("permission")
                    .and_then(Value::as_i64)
                    .unwrap_or_default(),
            ))
    });

    let permission_count = rows.len() as i64;
    let user_count = rows
        .iter()
        .filter(|row| value_text(row.get("subjectType")) == "user")
        .count() as i64;
    let team_count = rows
        .iter()
        .filter(|row| value_text(row.get("subjectType")) == "team")
        .count() as i64;
    let service_account_count = rows
        .iter()
        .filter(|row| value_text(row.get("subjectType")) == "service-account")
        .count() as i64;
    let role_count = rows
        .iter()
        .filter(|row| value_text(row.get("subjectType")) == "role")
        .count() as i64;

    let mut summary = Map::new();
    summary.insert("permissionCount".to_string(), Value::from(permission_count));
    summary.insert("userCount".to_string(), Value::from(user_count));
    summary.insert("teamCount".to_string(), Value::from(team_count));
    summary.insert(
        "serviceAccountCount".to_string(),
        Value::from(service_account_count),
    );
    summary.insert("roleCount".to_string(), Value::from(role_count));

    let mut resource = Map::new();
    resource.insert("kind".to_string(), Value::String(resource_kind.to_string()));
    resource.insert("uid".to_string(), Value::String(resource_uid.to_string()));
    resource.insert(
        "title".to_string(),
        Value::String(resource_title.to_string()),
    );

    let mut document = Map::new();
    document.insert(
        "kind".to_string(),
        Value::String(PERMISSION_EXPORT_KIND.to_string()),
    );
    document.insert(
        "schemaVersion".to_string(),
        Value::from(PERMISSION_EXPORT_SCHEMA_VERSION),
    );
    document.insert("resource".to_string(), Value::Object(resource));
    document.insert("summary".to_string(), Value::Object(summary));
    document.insert(
        "permissions".to_string(),
        Value::Array(rows.into_iter().map(Value::Object).collect()),
    );
    document.insert("org".to_string(), Value::String(org_name.to_string()));
    document.insert("orgId".to_string(), Value::String(org_id.to_string()));
    Value::Object(document)
}

pub(crate) fn build_permission_bundle_document(permission_documents: &[Value]) -> Value {
    let resource_count = permission_documents.len() as i64;
    let dashboard_count = permission_documents
        .iter()
        .filter(|item| {
            item.get("resource")
                .and_then(Value::as_object)
                .and_then(|resource| resource.get("kind"))
                .and_then(Value::as_str)
                == Some("dashboard")
        })
        .count() as i64;
    let folder_count = permission_documents
        .iter()
        .filter(|item| {
            item.get("resource")
                .and_then(Value::as_object)
                .and_then(|resource| resource.get("kind"))
                .and_then(Value::as_str)
                == Some("folder")
        })
        .count() as i64;
    let permission_count = permission_documents
        .iter()
        .map(|item| {
            item.get("summary")
                .and_then(Value::as_object)
                .and_then(|summary| summary.get("permissionCount"))
                .and_then(Value::as_i64)
                .unwrap_or_default()
        })
        .sum::<i64>();

    let mut summary = Map::new();
    summary.insert("resourceCount".to_string(), Value::from(resource_count));
    summary.insert("dashboardCount".to_string(), Value::from(dashboard_count));
    summary.insert("folderCount".to_string(), Value::from(folder_count));
    summary.insert("permissionCount".to_string(), Value::from(permission_count));

    let mut bundle = Map::new();
    bundle.insert(
        "kind".to_string(),
        Value::String(PERMISSION_BUNDLE_KIND.to_string()),
    );
    bundle.insert(
        "schemaVersion".to_string(),
        Value::from(PERMISSION_BUNDLE_SCHEMA_VERSION),
    );
    bundle.insert("summary".to_string(), Value::Object(summary));
    bundle.insert(
        "resources".to_string(),
        Value::Array(permission_documents.to_vec()),
    );
    Value::Object(bundle)
}
