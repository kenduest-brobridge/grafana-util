use super::{
    add_team_with_request, build_team_import_dry_run_document, diff_teams_with_request,
    export_teams_with_request, import_teams_with_request, list_teams_command_with_request,
    modify_team_with_request, DryRunOutputFormat, TeamAddArgs, TeamDiffArgs, TeamExportArgs,
    TeamImportArgs, TeamListArgs, TeamModifyArgs,
};
use crate::common::TOOL_VERSION;
use reqwest::Method;
use serde_json::{json, Value};
use std::fs;
use tempfile::tempdir;

#[test]
fn team_list_with_request_reads_search_and_members() {
    let args = TeamListArgs {
        common: super::make_token_common(),
        query: Some("ops".to_string()),
        name: None,
        with_members: true,
        output_columns: Vec::new(),
        list_columns: false,
        page: 1,
        per_page: 100,
        input_dir: None,
        local: false,
        run: None,
        run_id: None,
        table: false,
        csv: false,
        json: false,
        yaml: true,
        output_format: None,
    };
    let mut calls = Vec::new();
    let result = list_teams_command_with_request(
        |method, path, params, _payload| {
            calls.push((method.to_string(), path.to_string(), params.to_vec()));
            match path {
                "/api/teams/search" => Ok(Some(json!({
                    "teams": [{"id": 5, "name": "Ops", "memberCount": 1}]
                }))),
                "/api/teams/5/members" => Ok(Some(json!([{"login": "alice"}]))),
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    );

    assert_eq!(result.unwrap(), 1);
    assert!(calls.iter().any(|(_, path, _)| path == "/api/teams/search"));
    assert!(calls
        .iter()
        .any(|(_, path, _)| path == "/api/teams/5/members"));
}

#[test]
fn team_list_all_output_columns_are_accepted() {
    let args = TeamListArgs {
        common: super::make_token_common(),
        query: None,
        name: None,
        with_members: false,
        output_columns: vec!["all".to_string()],
        list_columns: false,
        page: 1,
        per_page: 100,
        input_dir: None,
        local: false,
        run: None,
        run_id: None,
        table: true,
        csv: false,
        json: false,
        yaml: false,
        output_format: None,
    };

    let result = list_teams_command_with_request(
        |method, path, _params, _payload| match (method, path) {
            (Method::GET, "/api/teams/search") => Ok(Some(json!({
                "teams": [{"id": 5, "name": "Ops", "memberCount": 1}]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    );

    assert_eq!(result.unwrap(), 1);
}

#[test]
fn team_add_with_request_creates_team_and_members() {
    let args = TeamAddArgs {
        common: super::make_token_common(),
        name: "Ops".to_string(),
        email: Some("ops@example.com".to_string()),
        members: vec!["alice@example.com".to_string()],
        admins: vec!["bob@example.com".to_string()],
        json: true,
    };
    let mut calls = Vec::new();
    let result = add_team_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match path {
                "/api/teams" => Ok(Some(json!({"teamId": 3}))),
                "/api/teams/3" => Ok(Some(
                    json!({"id": 3, "name": "Ops", "email": "ops@example.com"}),
                )),
                "/api/teams/3/members" if method == Method::POST => {
                    Ok(Some(json!({"message": "ok"})))
                }
                "/api/teams/3/members" if method == Method::GET => Ok(Some(json!([
                    {"login": "alice@example.com", "email": "alice@example.com", "userId": 7, "isAdmin": false}
                ]))),
                "/api/org/users" => Ok(Some(json!([
                    {"userId": 7, "login": "alice@example.com", "email": "alice@example.com"},
                    {"userId": 8, "login": "bob@example.com", "email": "bob@example.com"}
                ]))),
                "/api/teams/3/members" if method == Method::PUT => {
                    Ok(Some(json!({"message": "ok"})))
                }
                _ => panic!("unexpected path {path} {method:?}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls.iter().any(|(_, path, _, _)| path == "/api/teams"));
    let member_post_payload = calls
        .iter()
        .find(|(method, path, _, _)| method == "POST" && path == "/api/teams/3/members")
        .and_then(|(_, _, _, payload)| payload.as_ref())
        .expect("expected add-member payload");
    assert_eq!(member_post_payload.get("userId"), Some(&json!(7)));
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "PUT" && path == "/api/teams/3/members"));
}

#[test]
fn team_add_with_request_creates_empty_team_without_members() {
    let args = TeamAddArgs {
        common: super::make_token_common(),
        name: "Ops".to_string(),
        email: Some("ops@example.com".to_string()),
        members: Vec::new(),
        admins: Vec::new(),
        json: false,
    };
    let mut calls = Vec::new();
    let result = add_team_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match path {
                "/api/teams" => Ok(Some(json!({"teamId": 3}))),
                "/api/teams/3" => Ok(Some(
                    json!({"id": 3, "name": "Ops", "email": "ops@example.com"}),
                )),
                _ => panic!("unexpected path {path} {method:?}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls.iter().any(|(_, path, _, _)| path == "/api/teams"));
    assert!(!calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/teams/3/members"));
}

#[test]
fn team_modify_with_request_updates_members_and_admins() {
    let args = TeamModifyArgs {
        common: super::make_token_common(),
        team_id: Some("3".to_string()),
        name: None,
        add_member: vec!["alice@example.com".to_string()],
        remove_member: vec![],
        add_admin: vec!["bob@example.com".to_string()],
        remove_admin: vec![],
        json: true,
    };
    let mut calls = Vec::new();
    let result = modify_team_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match path {
                "/api/teams/3" => Ok(Some(json!({"id": 3, "name": "Ops"}))),
                "/api/org/users" => Ok(Some(json!([
                    {"userId": 7, "login": "alice@example.com", "email": "alice@example.com"},
                    {"userId": 8, "login": "bob@example.com", "email": "bob@example.com"}
                ]))),
                "/api/teams/3/members" if method == Method::POST => {
                    Ok(Some(json!({"message": "ok"})))
                }
                "/api/teams/3/members" if method == Method::GET => Ok(Some(json!([
                    {"login": "alice@example.com", "email": "alice@example.com", "userId": 7, "isAdmin": false}
                ]))),
                "/api/teams/3/members" if method == Method::PUT => {
                    Ok(Some(json!({"message": "ok"})))
                }
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "PUT" && path == "/api/teams/3/members"));
}

#[test]
fn diff_teams_with_request_returns_expected_difference_count() {
    let temp = tempdir().unwrap();
    let diff_dir = temp.path().join("access-teams");
    fs::create_dir_all(&diff_dir).unwrap();
    fs::write(
        diff_dir.join("teams.json"),
        r#"
[
  {"name":"Ops","email":"ops@example.com"},
  {"name":"Dev","email":"dev@example.com"}
]
"#,
    )
    .unwrap();
    let args = TeamDiffArgs {
        common: super::make_token_common(),
        diff_dir: diff_dir.clone(),
        local: false,
        run: None,
        run_id: None,
    };
    let result = diff_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id":"3","name":"Ops","email":"ops-two@example.com"},
                    {"id":"5","name":"SRE","email":"sre@example.com"}
                ]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    )
    .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn team_export_with_request_writes_bundle_with_members_and_admins() {
    let temp_dir = tempdir().unwrap();
    let args = TeamExportArgs {
        common: super::make_token_common(),
        output_dir: temp_dir.path().to_path_buf(),
        run: None,
        run_id: None,
        overwrite: true,
        dry_run: false,
        with_members: true,
    };
    let result = export_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id": 3, "name": "Ops", "email": "ops@example.com", "memberCount": 2}
                ]
            }))),
            "/api/teams/3/members" => Ok(Some(json!([
                {"userId": 7, "login": "alice@example.com", "email": "alice@example.com", "isAdmin": false},
                {"userId": 8, "login": "bob@example.com", "email": "bob@example.com", "isAdmin": true}
            ]))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    );

    assert!(result.is_ok());
    let bundle: Value =
        serde_json::from_str(&fs::read_to_string(temp_dir.path().join("teams.json")).unwrap())
            .unwrap();
    assert_eq!(
        bundle.get("kind"),
        Some(&json!("grafana-utils-access-team-export-index"))
    );
    let records = bundle
        .get("records")
        .and_then(Value::as_array)
        .expect("expected team export records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].get("name"), Some(&json!("Ops")));
    assert_eq!(
        records[0].get("members"),
        Some(&json!(["alice@example.com", "bob@example.com"]))
    );
    assert_eq!(records[0].get("admins"), Some(&json!(["bob@example.com"])));
    let metadata = super::read_json_file(&temp_dir.path().join("export-metadata.json"));
    assert_eq!(
        metadata.get("kind"),
        Some(&json!("grafana-utils-access-team-export-index"))
    );
    assert_eq!(metadata.get("version"), Some(&json!(1)));
    assert_eq!(metadata.get("recordCount"), Some(&json!(1)));
    assert_eq!(
        metadata.get("sourceUrl"),
        Some(&json!("http://127.0.0.1:3000"))
    );
    assert_eq!(
        metadata.get("sourceDir"),
        Some(&json!(temp_dir.path().to_string_lossy().to_string()))
    );
    assert_eq!(metadata.get("metadataVersion"), Some(&json!(2)));
    assert_eq!(metadata.get("domain"), Some(&json!("access")));
    assert_eq!(metadata.get("resourceKind"), Some(&json!("teams")));
    assert_eq!(metadata.get("bundleKind"), Some(&json!("export-root")));
    assert_eq!(metadata["source"]["kind"], json!("live"));
    assert_eq!(metadata["capture"]["recordCount"], json!(1));
}

#[test]
fn team_import_rejects_kind_mismatch_and_future_version_bundle_contract() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-user-export-index",
            "version": 1,
            "records": []
        }))
        .unwrap(),
    )
    .unwrap();
    let args = TeamImportArgs {
        common: super::make_token_common(),
        input_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
        replace_existing: true,
        dry_run: true,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: false,
    };
    let error =
        import_teams_with_request(|_method, _path, _params, _payload| Ok(None), &args).unwrap_err();
    assert!(error.to_string().contains("Access import kind mismatch"));

    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-team-export-index",
            "version": 99,
            "records": []
        }))
        .unwrap(),
    )
    .unwrap();
    let error =
        import_teams_with_request(|_method, _path, _params, _payload| Ok(None), &args).unwrap_err();
    assert!(error
        .to_string()
        .contains("Unsupported access import version"));
}

#[test]
fn team_import_resolves_members_to_email_for_bulk_update() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-team-export-index",
            "version": 1,
            "records": [
                {"name": "Ops", "email": "ops@example.com", "members": ["alice", "bob@example.com"], "admins": ["bob@example.com"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = TeamImportArgs {
        common: super::make_token_common(),
        input_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
        replace_existing: true,
        dry_run: false,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: false,
    };
    let mut calls = Vec::new();
    let result = import_teams_with_request(
        |method, path, _params, payload| {
            calls.push((method.to_string(), path.to_string(), payload.cloned()));
            match (method.clone(), path) {
                (Method::GET, "/api/org/users") => Ok(Some(json!([
                    {"userId": 7, "login": "alice", "email": "alice@example.com"},
                    {"userId": 8, "login": "bob", "email": "bob@example.com"}
                ]))),
                (Method::GET, "/api/teams/search") => Ok(Some(json!({
                    "teams": [{"id": 3, "uid": "team-ops", "name": "Ops", "email": "ops@example.com"}]
                }))),
                (Method::GET, "/api/teams/3/members") => Ok(Some(json!([]))),
                (Method::POST, "/api/teams/3/members") => Ok(Some(json!({"message": "ok"}))),
                (Method::PUT, "/api/teams/3/members") => Ok(Some(json!({"message": "ok"}))),
                _ => panic!("unexpected path {path} {method:?}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    let bulk_payload = calls
        .iter()
        .find(|(method, path, _)| method == "PUT" && path == "/api/teams/3/members")
        .and_then(|(_, _, payload)| payload.as_ref())
        .expect("expected bulk membership payload");
    assert_eq!(
        bulk_payload.get("members"),
        Some(&json!(["alice@example.com"]))
    );
    assert_eq!(
        bulk_payload.get("admins"),
        Some(&json!(["bob@example.com"]))
    );
}

#[test]
fn team_import_blocks_bulk_update_when_identity_has_no_email() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-team-export-index",
            "version": 1,
            "records": [
                {"name": "Ops", "members": ["alice"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = TeamImportArgs {
        common: super::make_token_common(),
        input_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
        replace_existing: true,
        dry_run: false,
        table: false,
        json: false,
        output_format: DryRunOutputFormat::Text,
        yes: false,
    };
    let error = import_teams_with_request(
        |method, path, _params, _payload| match (method.clone(), path) {
            (Method::GET, "/api/org/users") => Ok(Some(json!([
                {"userId": 7, "login": "alice"}
            ]))),
            _ => panic!("unexpected path {path} {method:?}"),
        },
        &args,
    )
    .unwrap_err();

    assert!(error.to_string().contains("require user emails"));
}

#[test]
fn team_import_dry_run_blocks_provisioned_team_membership_updates() {
    let temp_dir = tempdir().unwrap();
    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-access-team-export-index",
            "version": 1,
            "records": [
                {"name": "Ops", "members": ["alice@example.com"]}
            ]
        }))
        .unwrap(),
    )
    .unwrap();
    let args = TeamImportArgs {
        common: super::make_token_common(),
        input_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
        replace_existing: true,
        dry_run: true,
        table: true,
        json: false,
        output_format: DryRunOutputFormat::Table,
        yes: false,
    };
    let mut calls = Vec::new();
    let result = import_teams_with_request(
        |method, path, _params, payload| {
            calls.push((method.to_string(), path.to_string(), payload.cloned()));
            match (method.clone(), path) {
                (Method::GET, "/api/org/users") => Ok(Some(json!([
                    {"userId": 7, "login": "alice", "email": "alice@example.com"}
                ]))),
                (Method::GET, "/api/teams/search") => Ok(Some(json!({
                    "teams": [{"id": 3, "uid": "team-ops", "name": "Ops", "isProvisioned": true}]
                }))),
                (Method::GET, "/api/teams/3/members") => Ok(Some(json!([]))),
                _ => panic!("unexpected path {path} {method:?}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls
        .iter()
        .all(|(method, path, _)| !(method == "PUT" && path == "/api/teams/3/members")));
}

#[test]
fn team_diff_with_request_reports_same_state_for_members_and_admins() {
    let temp_dir = tempdir().unwrap();
    let export_args = TeamExportArgs {
        common: super::make_token_common(),
        output_dir: temp_dir.path().to_path_buf(),
        run: None,
        run_id: None,
        overwrite: true,
        dry_run: false,
        with_members: false,
    };
    export_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id": 3, "name": "Ops", "email": "ops@example.com", "memberCount": 2}
                ]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &export_args,
    )
    .unwrap();
    let args = TeamDiffArgs {
        common: super::make_token_common(),
        diff_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
    };
    let result = diff_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id": 3, "name": "Ops", "email": "ops@example.com"}
                ]
            }))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    );

    assert_eq!(result.unwrap(), 0);
}

#[test]
fn team_diff_with_request_reports_membership_drift() {
    let temp_dir = tempdir().unwrap();
    let export_args = TeamExportArgs {
        common: super::make_token_common(),
        output_dir: temp_dir.path().to_path_buf(),
        run: None,
        run_id: None,
        overwrite: true,
        dry_run: false,
        with_members: true,
    };
    export_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id": 3, "name": "Ops", "email": "ops@example.com", "memberCount": 2}
                ]
            }))),
            "/api/teams/3/members" => Ok(Some(json!([
                {"userId": 7, "login": "alice@example.com", "email": "alice@example.com", "isAdmin": false},
                {"userId": 8, "login": "bob@example.com", "email": "bob@example.com", "isAdmin": true}
            ]))),
            _ => panic!("unexpected path {path}"),
        },
        &export_args,
    )
    .unwrap();
    let mut bundle: Value =
        serde_json::from_str(&fs::read_to_string(temp_dir.path().join("teams.json")).unwrap())
            .unwrap();
    if let Some(records) = bundle.get_mut("records").and_then(Value::as_array_mut) {
        if let Some(team) = records.get_mut(0).and_then(Value::as_object_mut) {
            team.insert(
                "members".to_string(),
                Value::Array(vec![Value::String("alice@example.com".to_string())]),
            );
            team.insert("admins".to_string(), Value::Array(Vec::new()));
        }
    }
    fs::write(
        temp_dir.path().join("teams.json"),
        serde_json::to_string_pretty(&bundle).unwrap(),
    )
    .unwrap();
    let args = TeamDiffArgs {
        common: super::make_token_common(),
        diff_dir: temp_dir.path().to_path_buf(),
        local: false,
        run: None,
        run_id: None,
    };
    let result = diff_teams_with_request(
        |_method, path, _params, _payload| match path {
            "/api/teams/search" => Ok(Some(json!({
                "teams": [
                    {"id": 3, "name": "Ops", "email": "ops@example.com"}
                ]
            }))),
            "/api/teams/3/members" => Ok(Some(json!([
                {"userId": 7, "login": "alice@example.com", "email": "alice@example.com", "isAdmin": false},
                {"userId": 8, "login": "bob@example.com", "email": "bob@example.com", "isAdmin": true}
            ]))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    );

    assert_eq!(result.unwrap(), 1);
}

#[test]
fn team_import_dry_run_document_reports_summary_and_rows() {
    let rows = vec![
        serde_json::from_value::<serde_json::Map<String, Value>>(json!({
            "index": "1",
            "identity": "Ops",
            "action": "remove-member",
            "detail": "would remove team member bob@example.com"
        }))
        .unwrap(),
        serde_json::from_value::<serde_json::Map<String, Value>>(json!({
            "index": "1",
            "identity": "Ops",
            "action": "updated",
            "detail": "would update team"
        }))
        .unwrap(),
    ];
    let document = build_team_import_dry_run_document(
        &rows,
        1,
        0,
        1,
        0,
        std::path::Path::new("/tmp/access-teams"),
    );

    assert_eq!(
        document.get("kind"),
        Some(&json!("grafana-utils-access-import-dry-run"))
    );
    assert_eq!(document.get("resourceKind"), Some(&json!("team")));
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
            .and_then(|summary| summary.get("created")),
        Some(&json!(0))
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
            .and_then(|summary| summary.get("skipped")),
        Some(&json!(0))
    );
    assert_eq!(
        document
            .get("summary")
            .and_then(|summary| summary.get("source")),
        Some(&json!("/tmp/access-teams"))
    );
    assert_eq!(
        document.get("rows").and_then(Value::as_array).map(Vec::len),
        Some(2)
    );
}
