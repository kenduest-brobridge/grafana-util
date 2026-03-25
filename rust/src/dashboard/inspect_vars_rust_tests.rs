use super::test_support::{
    extract_dashboard_variables, parse_cli_from, DashboardCommand, SimpleOutputFormat,
};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn parse_inspect_vars_args_supports_dashboard_url_only() {
    let args = match parse_cli_from([
        "grafana-util",
        "inspect-vars",
        "--dashboard-url",
        "https://grafana.example.com/d/infra-main/infra-overview?var-datasource=prom-main",
        "--output-format",
        "json",
        "--token",
        "secret",
    ])
    .command
    {
        DashboardCommand::InspectVars(args) => args,
        other => panic!("expected inspect-vars args, got {other:?}"),
    };

    assert_eq!(args.dashboard_uid, None);
    assert_eq!(
        args.dashboard_url.as_deref(),
        Some("https://grafana.example.com/d/infra-main/infra-overview?var-datasource=prom-main")
    );
    assert_eq!(args.output_format, Some(SimpleOutputFormat::Json));
    assert_eq!(args.vars_query, None);
}

#[test]
fn parse_inspect_vars_args_accepts_vars_query() {
    let args = match parse_cli_from([
        "grafana-util",
        "inspect-vars",
        "--dashboard-uid",
        "infra-main",
        "--vars-query",
        "var-datasource=prom-main&var-cluster=prod-a",
        "--token",
        "secret",
    ])
    .command
    {
        DashboardCommand::InspectVars(args) => args,
        other => panic!("expected inspect-vars args, got {other:?}"),
    };

    assert_eq!(args.dashboard_uid.as_deref(), Some("infra-main"));
    assert_eq!(
        args.vars_query.as_deref(),
        Some("var-datasource=prom-main&var-cluster=prod-a")
    );
}

#[test]
fn parse_inspect_vars_args_supports_output_file() {
    let args = match parse_cli_from([
        "grafana-util",
        "inspect-vars",
        "--dashboard-uid",
        "infra-main",
        "--output-file",
        "/tmp/inspect-vars.json",
        "--token",
        "secret",
    ])
    .command
    {
        DashboardCommand::InspectVars(args) => args,
        other => panic!("expected inspect-vars args, got {other:?}"),
    };

    assert_eq!(args.dashboard_uid.as_deref(), Some("infra-main"));
    assert_eq!(
        args.output_file,
        Some(PathBuf::from("/tmp/inspect-vars.json"))
    );
}

#[test]
fn extract_dashboard_variables_reads_current_and_options() {
    let dashboard = json!({
        "templating": {
            "list": [
                {
                    "name": "datasource",
                    "type": "datasource",
                    "label": "Datasource",
                    "query": "prometheus",
                    "current": {"text": "Prom Main", "value": "prom-main"},
                    "options": [
                        {"text": "Prom Main", "value": "prom-main"},
                        {"text": "Prom DR", "value": "prom-dr"}
                    ]
                },
                {
                    "name": "cluster",
                    "type": "query",
                    "label": "Cluster",
                    "datasource": {"uid": "prom-main", "type": "prometheus"},
                    "current": {"text": ["prod-a", "prod-b"], "value": ["prod-a", "prod-b"]},
                    "multi": true,
                    "includeAll": true,
                    "options": [{"text": "prod-a", "value": "prod-a"}]
                }
            ]
        }
    });

    let rows = extract_dashboard_variables(dashboard.as_object().unwrap()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "datasource");
    assert_eq!(rows[0].current, "Prom Main (prom-main)");
    assert_eq!(rows[0].option_count, 2);
    assert_eq!(rows[1].name, "cluster");
    assert_eq!(rows[1].datasource, "prom-main");
    assert_eq!(rows[1].current, "prod-a|prod-b");
    assert!(rows[1].multi);
    assert!(rows[1].include_all);
}
