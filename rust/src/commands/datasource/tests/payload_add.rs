use super::super::*;

#[test]
fn build_add_payload_keeps_optional_json_fields() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--uid",
        "prom-main",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--access",
        "proxy",
        "--datasource-url",
        "http://prometheus:9090",
        "--default",
        "--json-data",
        r#"{"httpMethod":"POST"}"#,
        "--secure-json-data",
        r#"{"httpHeaderValue1":"secret"}"#,
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let payload = build_add_payload(&add_args).unwrap();

    assert_eq!(
        payload,
        json!({
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "access": "proxy",
            "url": "http://prometheus:9090",
            "isDefault": true,
            "jsonData": {"httpMethod": "POST"},
            "secureJsonData": {"httpHeaderValue1": "secret"}
        })
    );
}

#[test]
fn build_add_payload_supports_datasource_auth_and_header_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--uid",
        "prom-main",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--datasource-url",
        "http://prometheus:9090",
        "--apply-supported-defaults",
        "--basic-auth",
        "--basic-auth-user",
        "metrics-user",
        "--basic-auth-password",
        "metrics-pass",
        "--user",
        "query-user",
        "--password",
        "query-pass",
        "--with-credentials",
        "--http-header",
        "X-Scope-OrgID=tenant-a",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let payload = build_add_payload(&add_args).unwrap();

    assert_eq!(payload["basicAuth"], json!(true));
    assert_eq!(payload["basicAuthUser"], json!("metrics-user"));
    assert_eq!(payload["user"], json!("query-user"));
    assert_eq!(payload["withCredentials"], json!(true));
    assert_eq!(payload["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(
        payload["jsonData"]["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(payload["jsonData"]["tlsSkipVerify"], json!(true));
    assert_eq!(
        payload["jsonData"]["serverName"],
        json!("prometheus.internal")
    );
    assert_eq!(
        payload["secureJsonData"]["basicAuthPassword"],
        json!("metrics-pass")
    );
    assert_eq!(payload["secureJsonData"]["password"], json!("query-pass"));
    assert_eq!(
        payload["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-a")
    );
}

#[test]
fn build_add_payload_resolves_secret_placeholders_into_secure_json_data() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--uid",
        "loki-main",
        "--name",
        "Loki Main",
        "--type",
        "loki",
        "--secure-json-data-placeholders",
        r#"{"basicAuthPassword":"${secret:loki-basic-auth}","httpHeaderValue1":"${secret:loki-tenant-token}"}"#,
        "--secret-values",
        r#"{"loki-basic-auth":"secret-value","loki-tenant-token":"tenant-token"}"#,
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let payload = build_add_payload(&add_args).unwrap();

    assert_eq!(
        payload["secureJsonData"]["basicAuthPassword"],
        json!("secret-value")
    );
    assert_eq!(
        payload["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-token")
    );
}

#[test]
fn build_add_payload_rejects_secret_values_without_placeholders() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Loki Main",
        "--type",
        "loki",
        "--secret-values",
        r#"{"loki-basic-auth":"secret-value"}"#,
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let error = build_add_payload(&add_args).unwrap_err().to_string();
    assert!(error.contains("--secret-values requires --secure-json-data-placeholders"));
}

#[test]
fn build_add_payload_rejects_missing_secret_values_with_visibility_summary() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Loki Main",
        "--type",
        "loki",
        "--secure-json-data-placeholders",
        r#"{"basicAuthPassword":"${secret:loki-basic-auth}","httpHeaderValue1":"${secret:loki-tenant-token}"}"#,
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let error = build_add_payload(&add_args).unwrap_err().to_string();

    assert!(error.contains("--secure-json-data-placeholders requires --secret-values"));
    assert!(error.contains("\"providerKind\":\"inline-placeholder-map\""));
    assert!(error.contains("\"provider\":{\"inputFlag\":\"--secret-values\""));
    assert!(error.contains("\"placeholderNames\":[\"loki-basic-auth\",\"loki-tenant-token\"]"));
    assert!(error.contains("\"secretFields\":[\"basicAuthPassword\",\"httpHeaderValue1\"]"));
}

#[test]
fn build_add_payload_rejects_basic_auth_password_without_user() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "add",
        "--name",
        "Prometheus Main",
        "--type",
        "prometheus",
        "--basic-auth-password",
        "metrics-pass",
    ]);
    let add_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
        _ => panic!("expected datasource add"),
    };

    let error = build_add_payload(&add_args).unwrap_err().to_string();
    assert!(error.contains("requires --basic-auth-user"));
}

#[test]
fn build_add_payload_merges_nested_json_data_override_from_shared_fixture() {
    for case in load_nested_json_data_merge_cases() {
        if case["operation"] != json!("add") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let add_args = match parsed.command {
            crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
            _ => panic!("expected datasource add"),
        };

        let payload = build_add_payload(&add_args).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn build_add_payload_preserves_explicit_secure_json_data_from_shared_fixture() {
    for case in load_secure_json_merge_cases() {
        if case["operation"] != json!("add") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let add_args = match parsed.command {
            crate::datasource::DatasourceGroupCommand::Add(inner) => inner,
            _ => panic!("expected datasource add"),
        };

        let payload = build_add_payload(&add_args).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}
