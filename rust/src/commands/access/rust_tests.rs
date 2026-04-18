//! Access domain test suite.
//! Validates CLI parsing/help text surfaces and handler contract behavior with stubbed
//! request closures.
use super::{
    cli_defs::AccessCliRoot,
    cli_defs::CommonCliArgsNoOrgId,
    org::{
        delete_org_with_request, diff_orgs_with_request, export_orgs_with_request,
        import_orgs_with_request, list_orgs_with_request, modify_org_with_request, org_csv_headers,
        org_summary_line, org_table_headers, org_table_rows,
    },
    parse_cli_from,
    pending_delete::{
        delete_service_account_token_with_request, delete_service_account_with_request,
        delete_team_with_request,
    },
    render::{user_summary_line, user_table_rows},
    run_access_cli_with_request,
    service_account::{
        add_service_account_token_with_request, add_service_account_with_request,
        diff_service_accounts_with_request, export_service_accounts_with_request,
        import_service_accounts_with_request, list_service_accounts_command_with_request,
    },
    team::{
        add_team_with_request, build_team_import_dry_run_document, diff_teams_with_request,
        export_teams_with_request, import_teams_with_request, list_teams_command_with_request,
        modify_team_with_request,
    },
    user::{
        add_user_with_request, annotate_user_account_scope, build_user_import_dry_run_document,
        delete_user_with_request, diff_users_with_request, export_users_with_request,
        import_users_with_request, list_users_with_request, modify_user_with_request,
    },
    AccessCommand, CommonCliArgs, DryRunOutputFormat, OrgCommand, OrgDeleteArgs, OrgDiffArgs,
    OrgExportArgs, OrgImportArgs, OrgListArgs, OrgModifyArgs, Scope, ServiceAccountAddArgs,
    ServiceAccountCommand, ServiceAccountDeleteArgs, ServiceAccountDiffArgs,
    ServiceAccountExportArgs, ServiceAccountImportArgs, ServiceAccountListArgs,
    ServiceAccountTokenAddArgs, ServiceAccountTokenCommand, ServiceAccountTokenDeleteArgs,
    TeamAddArgs, TeamCommand, TeamDeleteArgs, TeamDiffArgs, TeamExportArgs, TeamImportArgs,
    TeamListArgs, TeamModifyArgs, UserAddArgs, UserCommand, UserDeleteArgs, UserDiffArgs,
    UserExportArgs, UserImportArgs, UserListArgs, UserModifyArgs,
};
use crate::common::TOOL_VERSION;
use clap::{CommandFactory, Parser};
use reqwest::Method;
use serde_json::{json, Map, Value};
use std::fs;
use tempfile::tempdir;

fn render_access_subcommand_help(path: &[&str]) -> String {
    let mut command = AccessCliRoot::command();
    let mut current = &mut command;
    for segment in path {
        current = current
            .find_subcommand_mut(segment)
            .unwrap_or_else(|| panic!("missing access subcommand help for {segment}"));
    }
    let mut output = Vec::new();
    current.write_long_help(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn access_delete_help_mentions_prompt() {
    assert!(render_access_subcommand_help(&["user", "delete"]).contains("--prompt"));
    assert!(render_access_subcommand_help(&["team", "delete"]).contains("--prompt"));
    assert!(render_access_subcommand_help(&["org", "delete"]).contains("--prompt"));
    assert!(render_access_subcommand_help(&["service-account", "delete"]).contains("--prompt"));
    assert!(
        render_access_subcommand_help(&["service-account", "token", "delete"]).contains("--prompt")
    );
}

#[test]
fn parse_cli_supports_access_delete_prompt_flags() {
    let user_args = parse_cli_from(["grafana-util access", "user", "delete", "--prompt"]);
    match user_args.command {
        AccessCommand::User {
            command: UserCommand::Delete(inner),
        } => {
            assert!(inner.prompt);
            assert_eq!(inner.scope, None);
        }
        _ => panic!("expected access user delete"),
    }

    let org_args = parse_cli_from(["grafana-util access", "org", "delete", "--prompt"]);
    match org_args.command {
        AccessCommand::Org {
            command: OrgCommand::Delete(inner),
        } => assert!(inner.prompt),
        _ => panic!("expected access org delete"),
    }
}

fn make_token_common() -> CommonCliArgs {
    CommonCliArgs {
        profile: None,
        url: "http://127.0.0.1:3000".to_string(),
        api_token: Some("token".to_string()),
        username: None,
        password: None,
        prompt_password: false,
        prompt_token: false,
        org_id: None,
        timeout: 30,
        verify_ssl: false,
        insecure: false,
        ca_cert: None,
    }
}

fn make_basic_common() -> CommonCliArgs {
    CommonCliArgs {
        profile: None,
        url: "http://127.0.0.1:3000".to_string(),
        api_token: None,
        username: Some("admin".to_string()),
        password: Some("secret".to_string()),
        prompt_password: false,
        prompt_token: false,
        org_id: None,
        timeout: 30,
        verify_ssl: false,
        insecure: false,
        ca_cert: None,
    }
}

fn make_basic_common_no_org_id() -> CommonCliArgsNoOrgId {
    CommonCliArgsNoOrgId {
        profile: None,
        url: "http://127.0.0.1:3000".to_string(),
        api_token: None,
        username: Some("admin".to_string()),
        password: Some("secret".to_string()),
        prompt_password: false,
        prompt_token: false,
        timeout: 30,
        verify_ssl: false,
        insecure: false,
        ca_cert: None,
    }
}

fn read_json_file(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn load_access_bundle_contract_cases() -> Vec<Value> {
    serde_json::from_str::<Value>(include_str!(
        "../../../../fixtures/access_bundle_contract_cases.json"
    ))
    .unwrap()
    .get("cases")
    .and_then(Value::as_array)
    .cloned()
    .unwrap_or_default()
}

#[test]
fn org_list_table_rows_include_user_summaries_only_when_requested() {
    let rows = vec![Map::from_iter(vec![
        ("id".to_string(), json!("1")),
        ("name".to_string(), json!("Main Org.")),
        ("userCount".to_string(), json!("2")),
        (
            "users".to_string(),
            json!([
                {"userId": "7", "login": "alice", "email": "alice@example.com", "name": "Alice", "orgRole": "Admin"},
                {"userId": "8", "login": "bob", "email": "bob@example.com", "name": "Bob", "orgRole": "Viewer"}
            ]),
        ),
    ])];

    assert_eq!(org_table_headers(false), vec!["ID", "NAME", "USER_COUNT"]);
    assert_eq!(org_csv_headers(false), vec!["id", "name", "userCount"]);
    assert_eq!(
        org_table_rows(&rows, false),
        vec![vec![
            "1".to_string(),
            "Main Org.".to_string(),
            "2".to_string()
        ]]
    );

    assert_eq!(
        org_table_headers(true),
        vec!["ID", "NAME", "USER_COUNT", "USERS"]
    );
    assert_eq!(
        org_csv_headers(true),
        vec!["id", "name", "userCount", "users"]
    );
    assert_eq!(
        org_table_rows(&rows, true),
        vec![vec![
            "1".to_string(),
            "Main Org.".to_string(),
            "2".to_string(),
            "alice(Admin); bob(Viewer)".to_string()
        ]]
    );
}

#[test]
fn org_summary_line_includes_users_when_requested() {
    let row = Map::from_iter(vec![
        ("id".to_string(), json!("4")),
        ("name".to_string(), json!("Audit Org")),
        ("userCount".to_string(), json!("1")),
        (
            "users".to_string(),
            json!([{"userId": "9", "email": "audit@example.com", "orgRole": "Editor"}]),
        ),
    ]);

    assert_eq!(
        org_summary_line(&row, false),
        "id=4 name=Audit Org userCount=1"
    );
    assert_eq!(
        org_summary_line(&row, true),
        "id=4 name=Audit Org userCount=1 users=audit@example.com(Editor)"
    );
}

#[path = "access_cli_rust_tests.rs"]
mod access_cli_rust_tests;

#[path = "access_runtime_org_rust_tests.rs"]
mod access_runtime_org_rust_tests;

#[path = "access_service_account_org_rust_tests.rs"]
mod access_service_account_org_rust_tests;

#[path = "access_team_rust_tests.rs"]
mod access_team_rust_tests;

#[path = "access_user_rust_tests.rs"]
mod access_user_rust_tests;
