//! Governance document regression tests.
//! Keeps governance document assembly coverage separate from the large dashboard test file.

use super::test_support;
use serde_json::{json, Value};

#[test]
fn build_export_inspection_governance_document_groups_core_family_dependency_rows() {
    let cases = [
        (
            "search",
            &["elasticsearch", "opensearch"][..],
            "status:500 AND service.name:\"api\"",
            &["service.name", "status"][..],
        ),
        (
            "tracing",
            &["jaeger", "tempo", "zipkin"][..],
            "service.name:api AND span.name:checkout AND traceID:abc123",
            &["service.name", "span.name", "traceID"][..],
        ),
    ];

    for (family, datasource_types, query_text, measurements) in cases {
        let dashboard_uid = format!("{family}-main");
        let queries = datasource_types
            .iter()
            .enumerate()
            .map(|(index, datasource_type)| {
                let panel_id = (index + 1).to_string();
                let ref_id = ((b'A' + index as u8) as char).to_string();
                let datasource_name = format!("{datasource_type}-main");
                let datasource_uid = format!("{datasource_type}-uid");
                test_support::make_core_family_report_row(
                    dashboard_uid.as_str(),
                    &panel_id,
                    &ref_id,
                    &datasource_uid,
                    &datasource_name,
                    datasource_type,
                    family,
                    query_text,
                    measurements,
                )
            })
            .collect::<Vec<test_support::ExportInspectionQueryRow>>();
        let summary = test_support::ExportInspectionSummary {
            import_dir: "/tmp/raw".to_string(),
            export_org: None,
            export_org_id: None,
            dashboard_count: 1,
            folder_count: 1,
            panel_count: datasource_types.len(),
            query_count: datasource_types.len(),
            datasource_inventory_count: 0,
            orphaned_datasource_count: 0,
            mixed_dashboard_count: 0,
            folder_paths: Vec::new(),
            datasource_usage: Vec::new(),
            datasource_inventory: Vec::new(),
            orphaned_datasources: Vec::new(),
            mixed_dashboards: Vec::new(),
        };
        let report = test_support::ExportInspectionQueryReport {
            import_dir: "/tmp/raw".to_string(),
            summary: test_support::QueryReportSummary {
                dashboard_count: 1,
                panel_count: datasource_types.len(),
                query_count: datasource_types.len(),
                report_row_count: datasource_types.len(),
            },
            queries,
        };

        let document = test_support::build_export_inspection_governance_document(&summary, &report);

        assert_eq!(document.summary.dashboard_count, 1);
        assert_eq!(document.summary.query_record_count, datasource_types.len());
        assert_eq!(document.summary.datasource_family_count, 1);
        assert_eq!(document.summary.risk_record_count, 0);
        assert_eq!(document.datasource_families.len(), 1);
        assert_eq!(document.datasource_families[0].family, family);
        assert_eq!(
            document.datasource_families[0].datasource_types,
            datasource_types
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<String>>()
        );
        assert_eq!(document.dashboard_dependencies.len(), 1);
        assert_eq!(
            document.dashboard_dependencies[0].datasource_families,
            vec![family.to_string()]
        );
        assert_eq!(
            document.dashboard_dependencies[0].query_fields,
            vec!["query".to_string()]
        );
        assert!(document.dashboard_dependencies[0].metrics.is_empty());
        assert!(document.dashboard_dependencies[0].functions.is_empty());
        assert!(document.risk_records.is_empty());
    }
}

#[test]
fn build_export_inspection_governance_document_rolls_up_dashboard_dependency_analysis() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 2,
        query_count: 2,
        datasource_inventory_count: 1,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![test_support::DatasourceInventorySummary {
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus:9090".to_string(),
            is_default: "true".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 2,
            dashboard_count: 1,
        }],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };
    let mut query_a = test_support::make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    query_a.query_field = "expr".to_string();
    query_a.metrics = vec!["http_requests_total".to_string()];
    query_a.functions = vec!["rate".to_string()];
    query_a.measurements = vec!["job=\"grafana\"".to_string()];
    query_a.buckets = vec!["5m".to_string()];

    let mut query_b = test_support::make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["service.name"],
    );
    query_b.query_field = "query".to_string();
    query_b.metrics = vec![
        "http_requests_total".to_string(),
        "process_cpu_seconds_total".to_string(),
    ];
    query_b.functions = vec!["rate".to_string(), "sum".to_string()];
    query_b.measurements = vec!["service.name".to_string(), "job=\"grafana\"".to_string()];
    query_b.buckets = vec!["1h".to_string(), "5m".to_string()];

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![query_a, query_b],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let dependency_row = &document_json["dashboardDependencies"][0];

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(document.summary.query_record_count, 2);
    assert_eq!(document.summary.datasource_family_count, 1);
    assert_eq!(document.summary.datasource_coverage_count, 1);
    assert_eq!(document.summary.dashboard_datasource_edge_count, 1);
    assert_eq!(document.summary.datasource_risk_coverage_count, 0);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 1);
    assert_eq!(document.summary.risk_record_count, 1);
    assert_eq!(dependency_row["queryFields"], json!(["expr", "query"]));
    assert_eq!(
        dependency_row["metrics"],
        json!(["http_requests_total", "process_cpu_seconds_total"])
    );
    assert_eq!(dependency_row["functions"], json!(["rate", "sum"]));
    assert_eq!(
        dependency_row["measurements"],
        json!(["job=\"grafana\"", "service.name"])
    );
    assert_eq!(dependency_row["buckets"], json!(["1h", "5m"]));
    assert_eq!(dependency_row["datasourceCount"], Value::from(1));
    assert_eq!(dependency_row["datasourceFamilyCount"], Value::from(1));
    assert_eq!(dependency_row["datasourceFamilies"], json!(["prometheus"]));
}

#[test]
fn build_export_inspection_governance_document_surfaces_datasource_blast_radius_dashboards() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 2,
        query_count: 2,
        datasource_inventory_count: 1,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![test_support::DatasourceInventorySummary {
            uid: "prom-main".to_string(),
            name: "Prometheus Main".to_string(),
            datasource_type: "prometheus".to_string(),
            access: "proxy".to_string(),
            url: "http://prometheus:9090".to_string(),
            is_default: "true".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 2,
            dashboard_count: 1,
        }],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: Vec::new(),
    };
    let query_a = test_support::make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    let mut query_b = test_support::make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["job=\"grafana\""],
    );
    query_b.query_field = "expr".to_string();

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 2,
            query_count: 2,
            report_row_count: 2,
        },
        queries: vec![query_a, query_b],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let datasource_row = &document_json["datasources"][0];
    let datasource_families = document_json["datasourceFamilies"].as_array().unwrap();

    assert_eq!(document.summary.dashboard_count, 1);
    assert_eq!(document.summary.query_record_count, 2);
    assert_eq!(document.summary.datasource_family_count, 1);
    assert_eq!(document.summary.datasource_coverage_count, 1);
    assert_eq!(document.summary.dashboard_datasource_edge_count, 1);
    assert_eq!(document.summary.datasource_risk_coverage_count, 0);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 0);
    assert_eq!(document.summary.risk_record_count, 0);
    assert_eq!(datasource_row["dashboardUids"], json!(["core-main"]));
    assert_eq!(datasource_row["dashboardCount"], Value::from(1));
    assert_eq!(datasource_row["panelCount"], Value::from(2));
    assert_eq!(datasource_row["queryCount"], Value::from(2));
    assert_eq!(datasource_row["queryFields"], json!(["expr", "query"]));
    assert_eq!(datasource_families.len(), 1);
    let datasource_governance_row = &document_json["datasourceGovernance"][0];
    assert_eq!(
        datasource_governance_row["datasourceUid"],
        json!("prom-main")
    );
    assert_eq!(datasource_governance_row["riskKinds"], json!([]));
    assert_eq!(datasource_governance_row["mixedDashboardCount"], json!(0));

    let lines = test_support::render_governance_table_report("/tmp/raw", &document);
    let output = lines.join("\n");
    assert!(output.contains("DATASOURCES_WITH_RISKS"));
    assert!(output.contains("# Datasource Governance"));
    assert!(output.contains("RISK_KINDS"));
    assert!(output.contains("MIXED_DASHBOARDS"));
    assert!(output.contains("ORPHANED_DATASOURCES"));
    assert!(output.contains("DASHBOARD_UIDS"));
    assert!(output.contains("PANELS"));
    assert!(output.contains("core-main"));
}

#[test]
fn render_governance_table_report_displays_sections() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: None,
        export_org_id: None,
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 1,
        query_count: 1,
        datasource_inventory_count: 2,
        orphaned_datasource_count: 1,
        mixed_dashboard_count: 0,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![
            test_support::DatasourceInventorySummary {
                uid: "logs-main".to_string(),
                name: "Logs Main".to_string(),
                datasource_type: "loki".to_string(),
                access: "proxy".to_string(),
                url: "http://loki:3100".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 1,
                dashboard_count: 1,
            },
            test_support::DatasourceInventorySummary {
                uid: "unused-main".to_string(),
                name: "Unused Main".to_string(),
                datasource_type: "tempo".to_string(),
                access: "proxy".to_string(),
                url: "http://tempo:3200".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 0,
                dashboard_count: 0,
            },
        ],
        orphaned_datasources: vec![test_support::DatasourceInventorySummary {
            uid: "unused-main".to_string(),
            name: "Unused Main".to_string(),
            datasource_type: "tempo".to_string(),
            access: "proxy".to_string(),
            url: "http://tempo:3200".to_string(),
            is_default: "false".to_string(),
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            reference_count: 0,
            dashboard_count: 0,
        }],
        mixed_dashboards: Vec::new(),
    };
    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 1,
            query_count: 1,
            report_row_count: 1,
        },
        queries: vec![test_support::ExportInspectionQueryRow {
            org: "Main Org.".to_string(),
            org_id: "1".to_string(),
            dashboard_uid: "logs-main".to_string(),
            dashboard_title: "Logs Main".to_string(),
            dashboard_tags: Vec::new(),
            folder_path: "Logs".to_string(),
            folder_full_path: "/Logs".to_string(),
            folder_level: "1".to_string(),
            folder_uid: "logs".to_string(),
            parent_folder_uid: String::new(),
            panel_id: "11".to_string(),
            panel_title: "Errors".to_string(),
            panel_type: "logs".to_string(),
            panel_target_count: 0,
            panel_query_count: 0,
            panel_datasource_count: 0,
            panel_variables: Vec::new(),
            ref_id: "A".to_string(),
            datasource: "logs-main".to_string(),
            datasource_name: "logs-main".to_string(),
            datasource_uid: "logs-main".to_string(),
            datasource_org: String::new(),
            datasource_org_id: String::new(),
            datasource_database: String::new(),
            datasource_bucket: String::new(),
            datasource_organization: String::new(),
            datasource_index_pattern: String::new(),
            datasource_type: "loki".to_string(),
            datasource_family: "loki".to_string(),
            query_field: "expr".to_string(),
            target_hidden: "false".to_string(),
            target_disabled: "false".to_string(),
            query_text: "{job=\"grafana\"}".to_string(),
            query_variables: Vec::new(),
            metrics: Vec::new(),
            functions: vec!["count_over_time".to_string()],
            measurements: vec!["job=\"grafana\"".to_string()],
            buckets: vec!["5m".to_string()],
            file_path: "/tmp/raw/logs-main.json".to_string(),
        }],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let lines = test_support::render_governance_table_report("/tmp/raw", &document);
    let output = lines.join("\n");

    assert!(output.contains("Export inspection governance: /tmp/raw"));
    assert!(output.contains("# Summary"));
    assert!(output.contains("# Datasource Families"));
    assert!(output.contains("# Dashboard Dependencies"));
    assert!(output.contains("# Dashboard Governance"));
    assert!(output.contains("# Dashboard Datasource Edges"));
    assert!(output.contains("# Datasource Governance"));
    assert!(output.contains("# Datasources"));
    assert!(output.contains("# Risks"));
    assert!(output.contains("DASHBOARD_UID"));
    assert!(output.contains("QUERY_FIELDS"));
    assert!(output.contains("DATASOURCE_COUNT"));
    assert!(output.contains("DATASOURCE_FAMILY_COUNT"));
    assert!(output.contains("DASHBOARD_DATASOURCE_EDGES"));
    assert!(output.contains("DATASOURCES_WITH_RISKS"));
    assert!(output.contains("DASHBOARDS_WITH_RISKS"));
    assert!(output.contains("METRICS"));
    assert!(output.contains("FUNCTIONS"));
    assert!(output.contains("MEASUREMENTS"));
    assert!(output.contains("BUCKETS"));
    assert!(output.contains("MIXED_DATASOURCE"));
    assert!(output.contains("RISK_KINDS"));
    assert!(output.contains("/tmp/raw/logs-main.json"));
    assert!(output.contains("CATEGORY"));
    assert!(output.contains("RECOMMENDATION"));
    assert!(output.contains("logs-main"));
    assert!(output.contains("unused-main"));
    assert!(output.contains("MIXED_DASHBOARDS"));
    assert!(output.contains("ORPHANED_DATASOURCES"));
    assert!(output.contains("orphaned-datasource"));
    assert!(output.contains("Remove the unused datasource"));
}

#[test]
fn build_export_inspection_governance_document_adds_dashboard_datasource_edges() {
    let summary = test_support::ExportInspectionSummary {
        import_dir: "/tmp/raw".to_string(),
        export_org: Some("Main Org.".to_string()),
        export_org_id: Some("1".to_string()),
        dashboard_count: 1,
        folder_count: 1,
        panel_count: 3,
        query_count: 3,
        datasource_inventory_count: 2,
        orphaned_datasource_count: 0,
        mixed_dashboard_count: 1,
        folder_paths: Vec::new(),
        datasource_usage: Vec::new(),
        datasource_inventory: vec![
            test_support::DatasourceInventorySummary {
                uid: "prom-main".to_string(),
                name: "Prometheus Main".to_string(),
                datasource_type: "prometheus".to_string(),
                access: "proxy".to_string(),
                url: "http://prometheus:9090".to_string(),
                is_default: "true".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 2,
                dashboard_count: 1,
            },
            test_support::DatasourceInventorySummary {
                uid: "logs-main".to_string(),
                name: "Logs Main".to_string(),
                datasource_type: "loki".to_string(),
                access: "proxy".to_string(),
                url: "http://loki:3100".to_string(),
                is_default: "false".to_string(),
                org: "Main Org.".to_string(),
                org_id: "1".to_string(),
                reference_count: 1,
                dashboard_count: 1,
            },
        ],
        orphaned_datasources: Vec::new(),
        mixed_dashboards: vec![test_support::MixedDashboardSummary {
            uid: "core-main".to_string(),
            title: "Core Main".to_string(),
            folder_path: "Platform".to_string(),
            datasource_count: 2,
            datasources: vec!["prom-main".to_string(), "logs-main".to_string()],
        }],
    };

    let mut prom_a = test_support::make_core_family_report_row(
        "core-main",
        "7",
        "A",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(http_requests_total[5m]))",
        &["job=\"grafana\""],
    );
    prom_a.query_field = "expr".to_string();
    prom_a.metrics = vec!["http_requests_total".to_string()];
    prom_a.functions = vec!["rate".to_string(), "sum".to_string()];
    prom_a.measurements = vec!["job=\"grafana\"".to_string()];
    prom_a.buckets = vec!["5m".to_string()];

    let mut prom_b = test_support::make_core_family_report_row(
        "core-main",
        "8",
        "B",
        "prom-main",
        "Prometheus Main",
        "prometheus",
        "prometheus",
        "sum(rate(process_cpu_seconds_total[1h]))",
        &["service.name"],
    );
    prom_b.query_field = "query".to_string();
    prom_b.metrics = vec!["process_cpu_seconds_total".to_string()];
    prom_b.functions = vec!["rate".to_string(), "sum".to_string()];
    prom_b.measurements = vec!["service.name".to_string()];
    prom_b.buckets = vec!["1h".to_string()];

    let mut loki = test_support::make_core_family_report_row(
        "core-main",
        "9",
        "C",
        "logs-main",
        "Logs Main",
        "loki",
        "loki",
        "{job=\"grafana\"} |= \"error\"",
        &["job=\"grafana\""],
    );
    loki.query_field = "expr".to_string();
    loki.functions = vec!["line_filter_contains".to_string()];
    loki.measurements = vec!["job=\"grafana\"".to_string()];

    let report = test_support::ExportInspectionQueryReport {
        import_dir: "/tmp/raw".to_string(),
        summary: test_support::QueryReportSummary {
            dashboard_count: 1,
            panel_count: 3,
            query_count: 3,
            report_row_count: 3,
        },
        queries: vec![prom_a, prom_b, loki],
    };

    let document = test_support::build_export_inspection_governance_document(&summary, &report);
    let document_json = serde_json::to_value(&document).unwrap();
    let edges = document_json["dashboardDatasourceEdges"]
        .as_array()
        .unwrap();
    let datasource_governance = document_json["datasourceGovernance"].as_array().unwrap();

    assert_eq!(document.summary.dashboard_datasource_edge_count, 2);
    assert_eq!(document.summary.datasource_risk_coverage_count, 2);
    assert_eq!(document.summary.dashboard_risk_coverage_count, 1);
    assert_eq!(edges.len(), 2);
    assert_eq!(datasource_governance.len(), 2);
    let dashboard_governance = document_json["dashboardGovernance"].as_array().unwrap();
    assert_eq!(dashboard_governance.len(), 1);

    let prom_edge = edges
        .iter()
        .find(|row| row["datasourceUid"] == json!("prom-main"))
        .unwrap();
    assert_eq!(prom_edge["dashboardUid"], json!("core-main"));
    assert_eq!(prom_edge["dashboardTitle"], json!("core-main Dashboard"));
    assert_eq!(prom_edge["panelCount"], json!(2));
    assert_eq!(prom_edge["queryCount"], json!(2));
    assert_eq!(prom_edge["queryFields"], json!(["expr", "query"]));
    assert_eq!(
        prom_edge["metrics"],
        json!(["http_requests_total", "process_cpu_seconds_total"])
    );
    assert_eq!(prom_edge["functions"], json!(["rate", "sum"]));
    assert_eq!(
        prom_edge["measurements"],
        json!(["job=\"grafana\"", "service.name"])
    );
    assert_eq!(prom_edge["buckets"], json!(["1h", "5m"]));

    let loki_edge = edges
        .iter()
        .find(|row| row["datasourceUid"] == json!("logs-main"))
        .unwrap();
    assert_eq!(loki_edge["family"], json!("loki"));
    assert_eq!(loki_edge["panelCount"], json!(1));
    assert_eq!(loki_edge["queryCount"], json!(1));
    assert_eq!(loki_edge["functions"], json!(["line_filter_contains"]));

    let prom_governance = datasource_governance
        .iter()
        .find(|row| row["datasourceUid"] == json!("prom-main"))
        .unwrap();
    assert_eq!(prom_governance["mixedDashboardCount"], json!(1));
    assert_eq!(
        prom_governance["riskKinds"],
        json!(["mixed-datasource-dashboard"])
    );

    let loki_governance = datasource_governance
        .iter()
        .find(|row| row["datasourceUid"] == json!("logs-main"))
        .unwrap();
    assert_eq!(
        loki_governance["riskKinds"],
        json!(["mixed-datasource-dashboard"])
    );

    let dashboard_governance_row = &dashboard_governance[0];
    assert_eq!(dashboard_governance_row["dashboardUid"], json!("core-main"));
    assert_eq!(dashboard_governance_row["mixedDatasource"], json!(true));
    assert_eq!(
        dashboard_governance_row["riskKinds"],
        json!(["large-prometheus-range", "mixed-datasource-dashboard"])
    );
}
