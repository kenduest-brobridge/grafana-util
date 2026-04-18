use super::super::*;

#[test]
fn user_import_with_request_org_scope_requires_yes_for_team_removal() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("users.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-user-export-index",
            "version": 1,
            "records": [
                {"login": "alice", "email": "alice@example.com", "name": "Alice", "orgRole": "Editor", "teams": ["ops"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = UserImportArgs {
        common: make_basic_common(),
        input_dir: temp_dir.path().to_path_buf(),
        scope: Scope::Org,
        replace_existing: true,
        dry_run: false,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: false,
    };
    let err = import_users_with_request(
        |method, path, _params, _payload| match (method, path) {
            (Method::GET, "/api/org/users") => Ok(Some(json!([
                {"userId": "7", "login": "alice", "email": "alice@example.com", "name": "Alice", "role": "Viewer"}
            ]))),
            (Method::GET, "/api/users/7/teams") => Ok(Some(json!([
                {"id": 12, "name": "legacy"}
            ]))),
            (Method::GET, "/api/teams/search") => Ok(Some(json!({
                "teams": [{"id": 11, "name": "ops"}]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    )
    .unwrap_err();

    assert!(err.to_string().contains("would remove team memberships"));
}

#[test]
fn user_import_with_request_dry_run_reports_org_role_and_team_drift() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("users.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-user-export-index",
            "version": 1,
            "records": [
                {"login": "alice", "email": "alice@example.com", "name": "Alice", "orgRole": "Editor", "teams": ["ops"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = UserImportArgs {
        common: make_basic_common(),
        input_dir: temp_dir.path().to_path_buf(),
        scope: Scope::Org,
        replace_existing: true,
        dry_run: true,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: true,
    };
    let mut calls = Vec::new();
    let result = import_users_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match (method, path) {
                (Method::GET, "/api/org/users") => Ok(Some(json!([
                    {"userId": "7", "login": "alice", "email": "alice@example.com", "name": "Alice", "role": "Viewer"}
                ]))),
                (Method::GET, "/api/users/7/teams") => Ok(Some(json!([
                    {"id": 12, "name": "legacy"}
                ]))),
                (Method::GET, "/api/teams/search") => Ok(Some(json!({
                    "teams": [{"id": 11, "name": "ops"}]
                }))),
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(result, 1);
    assert!(calls
        .iter()
        .all(|(method, path, _, _)| !(method == "PATCH" && path == "/api/org/users/7")));
    assert!(calls
        .iter()
        .all(|(method, path, _, _)| !(method == "POST" && path == "/api/teams/11/members")));
    assert!(calls
        .iter()
        .all(|(method, path, _, _)| !(method == "DELETE" && path == "/api/teams/12/members/7")));
}

#[test]
fn user_import_with_request_dry_run_json_reports_org_summary_and_rows() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("users.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-user-export-index",
            "version": 1,
            "records": [
                {"login": "alice", "email": "alice@example.com", "name": "Alice", "orgRole": "Editor", "teams": ["ops"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = UserImportArgs {
        common: make_basic_common(),
        input_dir: temp_dir.path().to_path_buf(),
        scope: Scope::Org,
        replace_existing: true,
        dry_run: true,
        table: false,
        json: true,
        output_format: DryRunOutputFormat::Json,
        yes: true,
    };

    let result = import_users_with_request(
        |method, path, _params, _payload| match (method, path) {
            (Method::GET, "/api/org/users") => Ok(Some(json!([
                {"userId": "7", "login": "alice", "email": "alice@example.com", "name": "Alice", "role": "Viewer"}
            ]))),
            (Method::GET, "/api/users/7/teams") => Ok(Some(json!([
                {"id": 12, "name": "legacy"}
            ]))),
            (Method::GET, "/api/teams/search") => Ok(Some(json!({
                "teams": [{"id": 11, "name": "ops"}]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    )
    .unwrap();

    assert_eq!(result, 0);
}

#[test]
fn user_import_with_request_updates_existing_org_user_role_and_team_membership() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("users.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-user-export-index",
            "version": 1,
            "records": [
                {"login": "alice", "email": "alice@example.com", "name": "Alice", "orgRole": "Editor", "teams": ["ops"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = UserImportArgs {
        common: make_basic_common(),
        input_dir: temp_dir.path().to_path_buf(),
        scope: Scope::Org,
        replace_existing: true,
        dry_run: false,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: true,
    };
    let mut calls = Vec::new();
    let result = import_users_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match (method, path) {
                (Method::GET, "/api/org/users") => Ok(Some(json!([
                    {"userId": "7", "login": "alice", "email": "alice@example.com", "name": "Alice", "role": "Viewer"}
                ]))),
                (Method::GET, "/api/users/7/teams") => Ok(Some(json!([
                    {"id": 12, "name": "legacy"}
                ]))),
                (Method::GET, "/api/teams/search") => Ok(Some(json!({
                    "teams": [{"id": 11, "name": "ops"}]
                }))),
                (Method::PATCH, "/api/org/users/7") => Ok(Some(json!({"message": "ok"}))),
                (Method::POST, "/api/teams/11/members") => Ok(Some(json!({"message": "ok"}))),
                (Method::DELETE, "/api/teams/12/members/7") => Ok(Some(json!({"message": "ok"}))),
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(result, 1);
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "PATCH" && path == "/api/org/users/7"));
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "POST" && path == "/api/teams/11/members"));
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "DELETE" && path == "/api/teams/12/members/7"));
}

#[test]
fn user_import_dry_run_document_reports_summary_and_rows() {
    let rows = vec![
        serde_json::from_value::<serde_json::Map<String, Value>>(json!({
            "index": "1",
            "identity": "alice",
            "action": "update-org-role",
            "detail": "would update orgRole -> Editor"
        }))
        .unwrap(),
        serde_json::from_value::<serde_json::Map<String, Value>>(json!({
            "index": "1",
            "identity": "alice",
            "action": "updated",
            "detail": "would update user"
        }))
        .unwrap(),
    ];
    let document = build_user_import_dry_run_document(
        &rows,
        1,
        0,
        1,
        0,
        std::path::Path::new("/tmp/access-users"),
    );

    assert_eq!(
        document.get("kind"),
        Some(&json!("grafana-utils-access-import-dry-run"))
    );
    assert_eq!(document.get("resourceKind"), Some(&json!("user")));
    assert_eq!(document.get("schemaVersion"), Some(&json!(1)));
    assert_eq!(document.get("toolVersion"), Some(&json!(TOOL_VERSION)));
    assert_eq!(document.get("reviewRequired"), Some(&json!(true)));
    assert_eq!(document.get("reviewed"), Some(&json!(false)));
    assert_eq!(
        document
            .get("summary")
            .and_then(|summary| summary.get("processed")),
        Some(&json!(1))
    );
    assert_eq!(
        document
            .get("summary")
            .and_then(|summary| summary.get("updated")),
        Some(&json!(1))
    );
    assert_eq!(
        document
            .get("summary")
            .and_then(|summary| summary.get("source")),
        Some(&json!("/tmp/access-users"))
    );
    assert_eq!(
        document.get("rows").and_then(Value::as_array).map(Vec::len),
        Some(2)
    );
}
