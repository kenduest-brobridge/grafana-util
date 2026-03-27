use crate::datasource_secret::{
    build_secret_placeholder_plan, collect_secret_placeholders, iter_secret_placeholder_names,
};
use serde_json::json;

#[test]
fn collect_secret_placeholders_rejects_raw_secret_values() {
    let secure_json_data = json!({
        "basicAuthPassword": "plain-text-secret"
    });
    let error = collect_secret_placeholders(secure_json_data.as_object())
        .unwrap_err()
        .to_string();

    assert!(error.contains("${secret:...} placeholders"));
}

#[test]
fn build_secret_placeholder_plan_shapes_review_summary() {
    let datasource = json!({
        "uid": "loki-main",
        "name": "Loki Main",
        "type": "loki",
        "secureJsonDataPlaceholders": {
            "basicAuthPassword": "${secret:loki-basic-auth}",
            "httpHeaderValue1": "${secret:loki-tenant-token}",
            "httpHeaderValue2": "${secret:loki-tenant-token}"
        }
    });
    let plan = build_secret_placeholder_plan(datasource.as_object().unwrap()).unwrap();

    assert_eq!(plan.datasource_uid.as_deref(), Some("loki-main"));
    assert_eq!(plan.placeholders.len(), 3);
    assert_eq!(
        iter_secret_placeholder_names(&plan.placeholders).collect::<Vec<_>>(),
        vec!["loki-basic-auth", "loki-tenant-token"]
    );
}
