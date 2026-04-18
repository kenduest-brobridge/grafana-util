use super::super::*;

#[test]
fn build_import_payload_matches_shared_contract_fixtures() {
    for case in load_contract_cases() {
        let object = case.as_object().unwrap();
        let normalized = object
            .get("expectedNormalizedRecord")
            .and_then(Value::as_object)
            .unwrap();
        let expected_payload = object.get("expectedImportPayload").cloned().unwrap();
        let record = DatasourceImportRecord {
            uid: normalized
                .get("uid")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            name: normalized
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            datasource_type: normalized
                .get("type")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            access: normalized
                .get("access")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            url: normalized
                .get("url")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            is_default: normalized.get("isDefault").and_then(Value::as_str).unwrap() == "true",
            org_name: normalized
                .get("org")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            org_id: normalized
                .get("orgId")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            basic_auth: None,
            basic_auth_user: String::new(),
            database: String::new(),
            json_data: None,
            read_only: None,
            version: None,
            api_version: None,
            secure_json_data_placeholders: None,
            user: String::new(),
            with_credentials: None,
        };

        assert_eq!(build_import_payload(&record), expected_payload);
    }
}

#[test]
fn datasource_import_record_round_trips_through_inventory_shape() {
    let record = DatasourceImportRecord {
        uid: "loki-main".to_string(),
        name: "Loki Logs".to_string(),
        datasource_type: "loki".to_string(),
        access: "proxy".to_string(),
        url: "http://loki:3100".to_string(),
        is_default: false,
        org_name: "Observability".to_string(),
        org_id: "7".to_string(),
        basic_auth: Some(true),
        basic_auth_user: "loki-user".to_string(),
        database: "logs".to_string(),
        json_data: Some(
            json!({
                "maxLines": 1000
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        read_only: None,
        version: None,
        api_version: None,
        secure_json_data_placeholders: Some(
            json!({
                "basicAuthPassword": "${secret:loki-main-basicauthpassword}"
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
        user: "query-user".to_string(),
        with_credentials: Some(true),
    };

    let inventory_record = record.to_inventory_record();
    let reparsed = DatasourceImportRecord::from_inventory_record(
        &inventory_record,
        "datasource inventory roundtrip test",
    )
    .unwrap();

    assert_eq!(reparsed, record);
}

#[test]
fn build_import_payload_resolves_secret_placeholders_into_secure_json_data() {
    let record = DatasourceImportRecord {
        uid: "loki-main".to_string(),
        name: "Loki Main".to_string(),
        datasource_type: "loki".to_string(),
        access: "proxy".to_string(),
        url: "http://loki:3100".to_string(),
        is_default: false,
        org_name: String::new(),
        org_id: "1".to_string(),
        basic_auth: None,
        basic_auth_user: String::new(),
        database: String::new(),
        json_data: None,
        read_only: None,
        version: None,
        api_version: None,
        secure_json_data_placeholders: json!({
            "basicAuthPassword": "${secret:loki-basic-auth}",
            "httpHeaderValue1": "${secret:loki-tenant-token}"
        })
        .as_object()
        .cloned(),
        user: String::new(),
        with_credentials: None,
    };

    let payload = build_import_payload_with_secret_values(
        &record,
        json!({
            "loki-basic-auth": "secret-value",
            "loki-tenant-token": "tenant-token"
        })
        .as_object(),
    )
    .unwrap();

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
fn build_import_payload_rejects_missing_secret_values_for_placeholders() {
    let record = DatasourceImportRecord {
        uid: "loki-main".to_string(),
        name: "Loki Main".to_string(),
        datasource_type: "loki".to_string(),
        access: "proxy".to_string(),
        url: "http://loki:3100".to_string(),
        is_default: false,
        org_name: String::new(),
        org_id: "1".to_string(),
        basic_auth: None,
        basic_auth_user: String::new(),
        database: String::new(),
        json_data: None,
        read_only: None,
        version: None,
        api_version: None,
        secure_json_data_placeholders: json!({
            "basicAuthPassword": "${secret:loki-basic-auth}"
        })
        .as_object()
        .cloned(),
        user: String::new(),
        with_credentials: None,
    };

    let error = build_import_payload_with_secret_values(&record, None)
        .unwrap_err()
        .to_string();
    assert!(error.contains("requires --secret-values"));
}

#[test]
fn build_datasource_import_dry_run_json_value_includes_secret_visibility() {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let input_dir = std::env::temp_dir().join(format!(
        "grafana-utils-datasource-secret-{}-{}",
        std::process::id(),
        unique_suffix
    ));
    std::fs::create_dir_all(&input_dir).unwrap();
    std::fs::write(
        input_dir.join(crate::datasource::EXPORT_METADATA_FILENAME),
        format!(
            "{{\n  \"schemaVersion\": {},\n  \"kind\": \"{}\",\n  \"variant\": \"root\",\n  \"resource\": \"datasource\",\n  \"datasourceCount\": 1,\n  \"datasourcesFile\": \"{}\",\n  \"indexFile\": \"index.json\",\n  \"format\": \"grafana-datasource-inventory-v1\"\n}}\n",
            1,
            "grafana-utils-datasource-export-index",
            crate::datasource::DATASOURCE_EXPORT_FILENAME
        ),
    )
    .unwrap();
    std::fs::write(
        input_dir.join(crate::datasource::DATASOURCE_EXPORT_FILENAME),
        r#"[
  {
    "uid": "loki-main",
    "name": "Loki Main",
    "type": "loki",
    "access": "proxy",
    "url": "http://loki:3100",
    "isDefault": false,
    "orgId": "1",
    "secureJsonDataPlaceholders": {
      "basicAuthPassword": "${secret:loki-basic-auth}",
      "httpHeaderValue1": "${secret:loki-tenant-token}"
    }
  }
]
"#,
    )
    .unwrap();

    let report = crate::datasource::DatasourceImportDryRunReport {
        mode: "create-or-update".to_string(),
        input_dir: input_dir.clone(),
        input_format: DatasourceImportInputFormat::Inventory,
        source_org_id: "1".to_string(),
        target_org_id: "7".to_string(),
        rows: vec![vec![
            "loki-main".to_string(),
            "Loki Main".to_string(),
            "loki".to_string(),
            "name".to_string(),
            "missing".to_string(),
            "would-create".to_string(),
            "7".to_string(),
            "datasources.json#0".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ]],
        datasource_count: 1,
        would_create: 1,
        would_update: 0,
        would_skip: 0,
        would_block: 0,
    };

    let value =
        crate::datasource::datasource_import_export::build_datasource_import_dry_run_json_value(
            &report,
        );

    assert_eq!(
        value["kind"],
        json!("grafana-util-datasource-import-dry-run")
    );
    assert_eq!(value["schemaVersion"], json!(1));
    assert!(value.get("toolVersion").is_some());
    assert_eq!(value["reviewRequired"], json!(true));
    assert_eq!(value["reviewed"], json!(false));
    assert_eq!(value["summary"]["secretVisibilityCount"], json!(1));
    assert_eq!(value["secretVisibility"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["secretVisibility"][0]["providerKind"],
        json!("inline-placeholder-map")
    );
    assert_eq!(
        value["secretVisibility"][0]["provider"]["inputFlag"],
        json!("--secret-values")
    );
    assert_eq!(
        value["secretVisibility"][0]["provider"]["placeholderNameStrategy"],
        json!("sanitize(<datasource-uid|name|type>-<secure-json-field>).lowercase")
    );
    assert_eq!(
        value["secretVisibility"][0]["placeholderNames"],
        json!(["loki-basic-auth", "loki-tenant-token"])
    );
    assert_eq!(
        value["secretVisibility"][0]["secretFields"],
        json!(["basicAuthPassword", "httpHeaderValue1"])
    );
    assert_eq!(value["datasources"][0]["matchBasis"], json!("name"));
    assert_eq!(value["datasources"][0]["targetUid"], Value::Null);
    assert_eq!(value["datasources"][0]["targetVersion"], Value::Null);
    assert_eq!(value["datasources"][0]["targetReadOnly"], Value::Null);
    assert_eq!(value["datasources"][0]["blockedReason"], Value::Null);
}
