use super::{
    build_contact_point_export_document, build_contact_point_output_path, build_import_operation,
    build_rule_export_document, build_rule_output_path, detect_document_kind, CONTACT_POINT_KIND, RULE_KIND,
};
use serde_json::json;
use std::path::Path;

#[test]
fn build_rule_output_path_keeps_folder_structure() {
    let rule = json!({
        "folderUID": "infra folder",
        "ruleGroup": "CPU Alerts",
        "title": "DB CPU > 90%",
        "uid": "rule-1",
    });
    let path = build_rule_output_path(Path::new("alerts/raw/rules"), rule.as_object().unwrap(), false);
    assert_eq!(
        path,
        Path::new("alerts/raw/rules/infra_folder/CPU_Alerts/DB_CPU_90__rule-1.json")
    );
}

#[test]
fn build_contact_point_output_path_uses_name_and_uid() {
    let contact_point = json!({
        "name": "Webhook Main",
        "uid": "cp-uid",
    });
    let path = build_contact_point_output_path(
        Path::new("alerts/raw/contact-points"),
        contact_point.as_object().unwrap(),
        false,
    );
    assert_eq!(
        path,
        Path::new("alerts/raw/contact-points/Webhook_Main/Webhook_Main__cp-uid.json")
    );
}

#[test]
fn build_rule_export_document_strips_server_managed_fields() {
    let document = build_rule_export_document(
        json!({
            "uid": "rule-uid",
            "title": "CPU High",
            "folderUID": "infra-folder",
            "ruleGroup": "cpu-alerts",
            "condition": "C",
            "data": [],
            "updated": "2026-03-10T10:00:00Z",
            "provenance": "api"
        })
        .as_object()
        .unwrap(),
    );
    assert_eq!(document["kind"], RULE_KIND);
    assert!(document["spec"].get("updated").is_none());
    assert!(document["spec"].get("provenance").is_none());
}

#[test]
fn detect_document_kind_accepts_plain_contact_point_shape() {
    let kind = detect_document_kind(
        json!({
            "name": "Webhook Main",
            "type": "webhook",
            "settings": {"url": "http://127.0.0.1/notify"}
        })
        .as_object()
        .unwrap(),
    )
    .unwrap();
    assert_eq!(kind, CONTACT_POINT_KIND);
}

#[test]
fn build_import_operation_accepts_plain_rule_document() {
    let (kind, payload) = build_import_operation(&json!({
        "uid": "rule-uid",
        "title": "CPU High",
        "folderUID": "infra-folder",
        "ruleGroup": "cpu-alerts",
        "condition": "C",
        "data": [],
    }))
    .unwrap();
    assert_eq!(kind, RULE_KIND);
    assert_eq!(payload["title"], "CPU High");
}

#[test]
fn build_contact_point_export_document_wraps_tool_document() {
    let document = build_contact_point_export_document(
        json!({
            "uid": "cp-uid",
            "name": "Webhook Main",
            "type": "webhook",
            "settings": {"url": "http://127.0.0.1/notify"},
            "provenance": "api"
        })
        .as_object()
        .unwrap(),
    );
    assert_eq!(document["kind"], CONTACT_POINT_KIND);
    assert!(document["spec"].get("provenance").is_none());
}
