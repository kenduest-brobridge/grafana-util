use super::*;
use crate::dashboard::DatasourceInventoryItem;
use serde_json::json;

#[test]
fn build_offline_dependency_contract_reports_unique_dashboard_and_panel_counts() {
    let document = build_offline_dependency_contract(
        &[
            json!({
                "dashboardUid": "dash-a",
                "dashboardTitle": "Dash A",
                "folderPath": "General",
                "panelId": "1",
                "panelTitle": "Panel One",
                "panelType": "timeseries",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceType": "prometheus",
                "file": "dash-a.json",
                "queryField": "expr",
                "query": "up"
            }),
            json!({
                "dashboardUid": "dash-a",
                "dashboardTitle": "Dash A",
                "folderPath": "General",
                "panelId": "1",
                "panelTitle": "Panel One",
                "panelType": "timeseries",
                "refId": "B",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceType": "prometheus",
                "file": "dash-a.json",
                "queryField": "expr",
                "query": "sum(rate(up[5m]))"
            }),
            json!({
                "dashboardUid": "dash-b",
                "dashboardTitle": "Dash B",
                "folderPath": "Operations",
                "panelId": "2",
                "panelTitle": "Panel Two",
                "panelType": "timeseries",
                "refId": "A",
                "datasource": "Prometheus Main",
                "datasourceUid": "prom-main",
                "datasourceType": "prometheus",
                "file": "dash-b.json",
                "queryField": "expr",
                "query": "rate(http_requests_total[5m])"
            }),
        ],
        &[DatasourceInventoryItem {
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: String::new(),
            url: String::new(),
            database: String::new(),
            default_bucket: String::new(),
            organization: String::new(),
            index_pattern: String::new(),
            is_default: String::new(),
            org: String::new(),
            org_id: String::new(),
        }],
    );

    assert_eq!(document["summary"]["queryCount"], json!(3));
    assert_eq!(document["summary"]["dashboardCount"], json!(2));
    assert_eq!(document["summary"]["panelCount"], json!(2));
    assert_eq!(document["summary"]["datasourceCount"], json!(1));
    assert_eq!(document["datasourceUsage"][0]["queryCount"], json!(3));
    assert_eq!(document["datasourceUsage"][0]["dashboardCount"], json!(2));
    assert_eq!(document["datasourceUsage"][0]["panelCount"], json!(2));
    assert_eq!(document["datasourceUsage"][0]["folderCount"], json!(2));
    assert_eq!(
        document["datasourceUsage"][0]["highBlastRadius"],
        json!(true)
    );
    assert_eq!(document["datasourceUsage"][0]["crossFolder"], json!(true));
    assert_eq!(
        document["datasourceUsage"][0]["folderPaths"],
        json!(["General", "Operations"])
    );
    assert_eq!(
        document["datasourceUsage"][0]["dashboardTitles"],
        json!(["Dash A", "Dash B"])
    );
    assert_eq!(
        document["datasourceUsage"][0]["queryFields"],
        json!(["expr"])
    );
}

#[test]
fn build_offline_dependency_contract_parses_loki_family_features() {
    let document = build_offline_dependency_contract(
        &[json!({
            "dashboardUid": "dash-loki",
            "dashboardTitle": "Loki Dash",
            "panelId": "7",
            "panelTitle": "Logs",
            "panelType": "logs",
            "refId": "A",
            "datasource": "Loki Main",
            "datasourceUid": "loki-main",
            "datasourceType": "loki",
            "file": "dash-loki.json",
            "queryField": "expr",
            "query": "{job=\"api\",namespace=~\"prod-.*\"} |= \"a|~b\" | line_format \"{{.message}}\" |~ \"timeout\" | json"
        })],
        &[DatasourceInventoryItem {
            uid: "loki-main".to_string(),
            name: "Loki Main".to_string(),
            datasource_type: "loki".to_string(),
            access: String::new(),
            url: String::new(),
            database: String::new(),
            default_bucket: String::new(),
            organization: String::new(),
            index_pattern: String::new(),
            is_default: String::new(),
            org: String::new(),
            org_id: String::new(),
        }],
    );

    let analysis = &document["queries"][0]["analysis"];
    let mut measurements = analysis["measurements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    measurements.sort();
    let mut functions = analysis["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    functions.sort();
    let mut labels = analysis["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    labels.sort();

    assert_eq!(
        measurements,
        vec![
            "job=\"api\"",
            "namespace=~\"prod-.*\"",
            "{job=\"api\",namespace=~\"prod-.*\"}"
        ]
    );
    assert_eq!(
        functions,
        vec![
            "json",
            "line_filter_contains",
            "line_filter_contains:a|~b",
            "line_filter_regex",
            "line_filter_regex:timeout",
            "line_format"
        ]
    );
    assert_eq!(labels, vec!["job=\"api\"", "namespace=~\"prod-.*\""]);
}

#[test]
fn parse_sql_features_extracts_shape_and_source() {
    let document = build_offline_dependency_contract(
        &[json!({
            "dashboardUid": "dash-sql",
            "dashboardTitle": "SQL Dash",
            "panelId": "3",
            "panelTitle": "SQL",
            "panelType": "table",
            "refId": "A",
            "datasource": "PG Main",
            "datasourceUid": "pg-main",
            "datasourceType": "postgres",
            "file": "dash-sql.json",
            "queryField": "rawQuery",
            "query": "WITH recent AS (SELECT id FROM users WHERE active = true) SELECT p.id, u.name FROM posts p JOIN users u ON p.user_id = u.id WHERE p.created_at > now() - INTERVAL '1 day' LIMIT 10"
        })],
        &[DatasourceInventoryItem {
            uid: "pg-main".to_string(),
            name: "PG Main".to_string(),
            datasource_type: "postgres".to_string(),
            access: String::new(),
            url: String::new(),
            database: String::new(),
            default_bucket: String::new(),
            organization: String::new(),
            index_pattern: String::new(),
            is_default: String::new(),
            org: String::new(),
            org_id: String::new(),
        }],
    );

    let analysis = &document["queries"][0]["analysis"];
    let mut functions = analysis["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    functions.sort();
    let mut measurements = analysis["measurements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    measurements.sort();

    assert!(functions.binary_search(&"with".to_string()).is_ok());
    assert!(functions.binary_search(&"select".to_string()).is_ok());
    assert!(functions.binary_search(&"join".to_string()).is_ok());
    assert!(functions.binary_search(&"limit".to_string()).is_ok());
    assert!(measurements.contains(&"posts".to_string()));
    assert!(measurements.contains(&"users".to_string()));
    assert!(!measurements.contains(&"recent".to_string()));
}

#[test]
fn build_offline_dependency_contract_surfaces_richer_usage_and_orphan_details() {
    let document = build_offline_dependency_contract(
        &[json!({
            "dashboardUid": "dash-a",
            "dashboardTitle": "Dash A",
            "folderPath": "General",
            "panelId": "7",
            "panelTitle": "Panel",
            "panelType": "timeseries",
            "refId": "A",
            "datasource": "Prometheus Main",
            "datasourceUid": "prom-main",
            "datasourceType": "prometheus",
            "file": "dash-a.json",
            "queryField": "expr",
            "query": "up"
        })],
        &[
            DatasourceInventoryItem {
                uid: "prom-main".to_string(),
                name: "Prometheus Main".to_string(),
                datasource_type: "prometheus".to_string(),
                access: "proxy".to_string(),
                url: "http://prometheus:9090".to_string(),
                database: String::new(),
                default_bucket: String::new(),
                organization: String::new(),
                index_pattern: String::new(),
                is_default: "true".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
            },
            DatasourceInventoryItem {
                uid: "orphan-main".to_string(),
                name: "Orphan Main".to_string(),
                datasource_type: "grafana-postgresql-datasource".to_string(),
                access: "proxy".to_string(),
                url: "postgresql://postgres:5432/orphan".to_string(),
                database: "orphan_db".to_string(),
                default_bucket: String::new(),
                organization: String::new(),
                index_pattern: String::new(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
            },
        ],
    );

    assert_eq!(document["summary"]["orphanedDatasourceCount"], json!(1));
    assert_eq!(
        document["datasourceUsage"][0]["datasourceUid"],
        json!("prom-main")
    );
    assert_eq!(
        document["datasourceUsage"][0]["datasourceType"],
        json!("prometheus")
    );
    assert_eq!(
        document["datasourceUsage"][0]["queryFields"],
        json!(["expr"])
    );
    assert_eq!(document["datasourceUsage"][0]["folderCount"], json!(1));
    assert_eq!(
        document["datasourceUsage"][0]["highBlastRadius"],
        json!(false)
    );
    assert_eq!(document["datasourceUsage"][0]["crossFolder"], json!(false));
    assert_eq!(
        document["datasourceUsage"][0]["folderPaths"],
        json!(["General"])
    );
    assert_eq!(
        document["datasourceUsage"][0]["dashboardTitles"],
        json!(["Dash A"])
    );
    assert_eq!(
        document["orphanedDatasources"][0]["uid"],
        json!("orphan-main")
    );
    assert_eq!(
        document["orphanedDatasources"][0]["name"],
        json!("Orphan Main")
    );
    assert_eq!(
        document["orphanedDatasources"][0]["family"],
        json!("postgresql")
    );
    assert_eq!(
        document["orphanedDatasources"][0]["database"],
        json!("orphan_db")
    );
}
