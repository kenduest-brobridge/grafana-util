use super::{normalize_compare_payload, CONTACT_POINT_KIND, POLICIES_KIND, RULE_KIND};
use serde_json::json;

#[test]
fn normalize_compare_payload_erases_authoring_round_trip_drift_defaults() {
    let contact_point = json!({
        "uid": "cp-authoring",
        "name": "Authoring Webhook",
        "type": "webhook",
        "settings": {"url": "http://127.0.0.1/notify"},
    });
    let contact_point_live = json!({
        "uid": "cp-authoring",
        "name": "Authoring Webhook",
        "type": "webhook",
        "settings": {"url": "http://127.0.0.1/notify"},
        "disableResolveMessage": false,
    });
    assert_eq!(
        normalize_compare_payload(CONTACT_POINT_KIND, contact_point.as_object().unwrap()),
        normalize_compare_payload(CONTACT_POINT_KIND, contact_point_live.as_object().unwrap())
    );

    let policies_desired = json!({
        "receiver": "pagerduty-primary",
        "group_by": ["alertname", "grafana_folder"],
        "routes": [{
            "receiver": "pagerduty-primary",
            "continue": false,
            "group_by": ["alertname", "grafana_folder"],
            "object_matchers": [
                ["team", "=", "platform"],
                ["severity", "=", "critical"],
                ["grafana_utils_route", "=", "pagerduty-primary"]
            ]
        }]
    });
    let policies_live = json!({
        "receiver": "pagerduty-primary",
        "group_by": ["grafana_folder", "alertname"],
        "routes": [{
            "receiver": "pagerduty-primary",
            "group_by": ["grafana_folder", "alertname"],
            "object_matchers": [
                ["grafana_utils_route", "=", "pagerduty-primary"],
                ["severity", "=", "critical"],
                ["team", "=", "platform"]
            ]
        }]
    });
    assert_eq!(
        normalize_compare_payload(POLICIES_KIND, policies_desired.as_object().unwrap()),
        normalize_compare_payload(POLICIES_KIND, policies_live.as_object().unwrap())
    );

    let rule_desired = json!({
        "uid": "cpu-high",
        "title": "cpu-high",
        "folderUID": "platform-alerts",
        "ruleGroup": "cpu",
        "condition": "A",
        "for": "5m",
        "noDataState": "NoData",
        "execErrState": "Alerting",
        "labels": {
            "grafana_utils_route": "pagerduty-primary",
            "severity": "critical",
            "team": "platform"
        },
        "annotations": {},
        "data": [{
            "refId": "A",
            "datasourceUid": "__expr__",
            "relativeTimeRange": {"from": 0, "to": 0},
            "model": {
                "refId": "A",
                "type": "classic_conditions",
                "datasource": {"type": "__expr__", "uid": "__expr__"},
                "expression": "A",
                "conditions": [{
                    "type": "query",
                    "query": {"params": ["A"]},
                    "reducer": {"type": "last", "params": []},
                    "evaluator": {"type": "gt", "params": [80.0]},
                    "operator": {"type": "and"}
                }],
                "intervalMs": 1000,
                "maxDataPoints": 43200
            }
        }]
    });
    let rule_live = json!({
        "uid": "cpu-high",
        "title": "cpu-high",
        "folderUID": "platform-alerts",
        "ruleGroup": "cpu",
        "condition": "A",
        "for": "5m",
        "noDataState": "NoData",
        "execErrState": "Alerting",
        "isPaused": false,
        "keep_firing_for": "0s",
        "notification_settings": null,
        "record": null,
        "orgID": 1,
        "labels": {
            "grafana_utils_route": "pagerduty-primary",
            "severity": "critical",
            "team": "platform"
        },
        "annotations": {},
        "data": [{
            "refId": "A",
            "queryType": "",
            "datasourceUid": "__expr__",
            "relativeTimeRange": {"from": 0, "to": 0},
            "model": {
                "refId": "A",
                "type": "classic_conditions",
                "datasource": {"type": "__expr__", "uid": "__expr__"},
                "expression": "A",
                "conditions": [{
                    "type": "query",
                    "query": {"params": ["A"]},
                    "reducer": {"type": "last", "params": []},
                    "evaluator": {"type": "gt", "params": [80.0]},
                    "operator": {"type": "and"}
                }],
                "intervalMs": 1000,
                "maxDataPoints": 43200
            }
        }]
    });
    assert_eq!(
        normalize_compare_payload(RULE_KIND, rule_desired.as_object().unwrap()),
        normalize_compare_payload(RULE_KIND, rule_live.as_object().unwrap())
    );

    let rule_desired_alias = json!({
        "uid": "cpu-high",
        "title": "cpu-high",
        "folderUID": "platform-alerts",
        "ruleGroup": "cpu",
        "condition": "A",
        "for": "5m",
        "keep_firing_for": "0s",
        "noDataState": "NoData",
        "execErrState": "Alerting",
        "labels": {
            "grafana_utils_route": "pagerduty-primary",
            "severity": "critical",
            "team": "platform"
        },
        "annotations": {},
        "data": [{
            "refId": "A",
            "datasourceUid": "__expr__",
            "relativeTimeRange": {"from": 0, "to": 0},
            "model": {
                "refId": "A",
                "type": "classic_conditions",
                "datasource": {"type": "__expr__", "uid": "__expr__"},
                "expression": "A",
                "conditions": [{
                    "type": "query",
                    "query": {"params": ["A"]},
                    "reducer": {"type": "last", "params": []},
                    "evaluator": {"type": "gt", "params": [80.0]},
                    "operator": {"type": "and"}
                }],
                "intervalMs": 1000,
                "maxDataPoints": 43200
            }
        }]
    });
    let rule_live_alias = json!({
        "uid": "cpu-high",
        "title": "cpu-high",
        "folderUID": "platform-alerts",
        "ruleGroup": "cpu",
        "condition": "A",
        "for": "300s",
        "keep_firing_for": "0m",
        "noDataState": "NoData",
        "execErrState": "Alerting",
        "labels": {
            "grafana_utils_route": "pagerduty-primary",
            "severity": "critical",
            "team": "platform"
        },
        "annotations": {},
        "data": [{
            "refId": "A",
            "queryType": "",
            "datasourceUid": "__expr__",
            "relativeTimeRange": {"from": 0, "to": 0},
            "model": {
                "refId": "A",
                "type": "classic_conditions",
                "datasource": {"type": "__expr__", "uid": "__expr__"},
                "expression": "A",
                "conditions": [{
                    "type": "query",
                    "query": {"params": ["A"]},
                    "reducer": {"type": "last", "params": []},
                    "evaluator": {"type": "gt", "params": [80.0]},
                    "operator": {"type": "and"}
                }],
                "intervalMs": 1000,
                "maxDataPoints": 43200
            }
        }]
    });
    assert_eq!(
        normalize_compare_payload(RULE_KIND, rule_desired_alias.as_object().unwrap()),
        normalize_compare_payload(RULE_KIND, rule_live_alias.as_object().unwrap())
    );
}
