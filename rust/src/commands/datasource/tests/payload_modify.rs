use super::super::*;

#[test]
fn build_modify_updates_keeps_optional_json_fields() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--set-url",
        "http://prometheus-v2:9090",
        "--set-access",
        "direct",
        "--set-default",
        "true",
        "--json-data",
        r#"{"httpMethod":"POST"}"#,
        "--secure-json-data",
        r#"{"token":"abc123"}"#,
    ]);
    let modify_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };

    let updates = build_modify_updates(&modify_args).unwrap();

    assert_eq!(updates["url"], json!("http://prometheus-v2:9090"));
    assert_eq!(updates["access"], json!("direct"));
    assert_eq!(updates["isDefault"], json!(true));
    assert_eq!(updates["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(updates["secureJsonData"]["token"], json!("abc123"));
}

#[test]
fn build_modify_updates_supports_datasource_auth_and_header_flags() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
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
        "X-Scope-OrgID=tenant-b",
        "--tls-skip-verify",
        "--server-name",
        "prometheus.internal",
    ]);
    let modify_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };

    let updates = build_modify_updates(&modify_args).unwrap();

    assert_eq!(updates["basicAuth"], json!(true));
    assert_eq!(updates["basicAuthUser"], json!("metrics-user"));
    assert_eq!(updates["user"], json!("query-user"));
    assert_eq!(updates["withCredentials"], json!(true));
    assert_eq!(
        updates["jsonData"]["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(updates["jsonData"]["tlsSkipVerify"], json!(true));
    assert_eq!(
        updates["jsonData"]["serverName"],
        json!("prometheus.internal")
    );
    assert_eq!(
        updates["secureJsonData"]["basicAuthPassword"],
        json!("metrics-pass")
    );
    assert_eq!(updates["secureJsonData"]["password"], json!("query-pass"));
    assert_eq!(
        updates["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-b")
    );
}

#[test]
fn build_modify_updates_resolves_secret_placeholders_into_secure_json_data() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "loki-main",
        "--secure-json-data-placeholders",
        r#"{"basicAuthPassword":"${secret:loki-basic-auth}","httpHeaderValue1":"${secret:loki-tenant-token}"}"#,
        "--secret-values",
        r#"{"loki-basic-auth":"secret-value","loki-tenant-token":"tenant-token"}"#,
    ]);
    let modify_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };

    let updates = build_modify_updates(&modify_args).unwrap();

    assert_eq!(
        updates["secureJsonData"]["basicAuthPassword"],
        json!("secret-value")
    );
    assert_eq!(
        updates["secureJsonData"]["httpHeaderValue1"],
        json!("tenant-token")
    );
}

#[test]
fn build_modify_payload_merges_existing_json_data() {
    let existing = json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "access": "proxy",
        "isDefault": false,
        "basicAuth": true,
        "basicAuthUser": "metrics-user",
        "user": "query-user",
        "withCredentials": true,
        "jsonData": {
            "httpMethod": "POST"
        }
    })
    .as_object()
    .unwrap()
    .clone();
    let updates = json!({
        "url": "http://prometheus-v2:9090",
        "jsonData": {
            "timeInterval": "30s"
        },
        "secureJsonData": {
            "token": "abc123"
        }
    })
    .as_object()
    .unwrap()
    .clone();

    let payload = build_modify_payload(&existing, &updates).unwrap();

    assert_eq!(payload["url"], json!("http://prometheus-v2:9090"));
    assert_eq!(payload["basicAuth"], json!(true));
    assert_eq!(payload["basicAuthUser"], json!("metrics-user"));
    assert_eq!(payload["user"], json!("query-user"));
    assert_eq!(payload["withCredentials"], json!(true));
    assert_eq!(payload["jsonData"]["httpMethod"], json!("POST"));
    assert_eq!(payload["jsonData"]["timeInterval"], json!("30s"));
    assert_eq!(payload["secureJsonData"]["token"], json!("abc123"));
}

#[test]
fn build_modify_payload_rejects_basic_auth_password_without_basic_auth_user() {
    let args = DatasourceCliArgs::parse_normalized_from([
        "grafana-util",
        "modify",
        "--uid",
        "prom-main",
        "--basic-auth-password",
        "metrics-pass",
    ]);
    let modify_args = match args.command {
        crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
        _ => panic!("expected datasource modify"),
    };
    let updates = build_modify_updates(&modify_args).unwrap();
    let existing = json!({
        "id": 7,
        "uid": "prom-main",
        "name": "Prometheus Main",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "access": "proxy",
        "isDefault": false
    })
    .as_object()
    .unwrap()
    .clone();

    let error = build_modify_payload(&existing, &updates).unwrap_err();

    assert!(error
        .to_string()
        .contains("--basic-auth-password requires --basic-auth-user or an existing basicAuthUser"));
}

#[test]
fn build_modify_payload_deep_merges_nested_json_data_override_from_shared_fixture() {
    for case in load_nested_json_data_merge_cases() {
        if case["operation"] != json!("modify") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let modify_args = match parsed.command {
            crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
            _ => panic!("expected datasource modify"),
        };

        let updates = build_modify_updates(&modify_args).unwrap();
        let existing = case["existing"].as_object().unwrap().clone();
        let payload = build_modify_payload(&existing, &updates).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}

#[test]
fn build_modify_payload_replaces_secure_json_data_from_shared_fixture() {
    for case in load_secure_json_merge_cases() {
        if case["operation"] != json!("modify") {
            continue;
        }
        let args = case["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>();
        let parsed = DatasourceCliArgs::parse_normalized_from(args);
        let modify_args = match parsed.command {
            crate::datasource::DatasourceGroupCommand::Modify(inner) => inner,
            _ => panic!("expected datasource modify"),
        };

        let updates = build_modify_updates(&modify_args).unwrap();
        let existing = case["existing"].as_object().unwrap().clone();
        let payload = build_modify_payload(&existing, &updates).unwrap();
        let expected = case["expected"].as_object().unwrap();

        assert_json_subset(&payload, &Value::Object(expected.clone()));
    }
}
