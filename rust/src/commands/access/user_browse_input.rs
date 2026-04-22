//! Interactive user browse row-loading boundary.

#[path = "user_browse_load.rs"]
mod user_browse_load;
pub(crate) use self::user_browse_load::load_rows;

#[cfg(test)]
mod tests {
    use super::super::user_browse_key::handle_key;
    use super::super::user_browse_state::{row_kind, BrowserState, DisplayMode};
    use super::*;
    use crate::access::{CommonCliArgs, Scope, UserBrowseArgs};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use reqwest::Method;
    use serde_json::Map;
    use serde_json::Value;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_rows_reads_local_user_bundle_without_live_requests() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("users.json"),
            r#"{
                "kind":"grafana-utils-access-user-export-index",
                "version":1,
                "records":[
                    {"login":"alice","email":"alice@example.com","name":"Alice","orgRole":"Editor","scope":"org","teams":["ops","sre"]},
                    {"login":"bob","email":"bob@example.com","name":"Bob","scope":"global","teams":["platform"]}
                ]
            }"#,
        )
        .unwrap();
        let args = UserBrowseArgs {
            common: CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            input_dir: Some(temp.path().to_path_buf()),
            local: false,
            run: None,
            run_id: None,
            scope: Scope::Org,
            all_orgs: false,
            current_org: false,
            query: None,
            login: Some("alice".to_string()),
            email: None,
            org_role: None,
            grafana_admin: None,
            with_teams: false,
            page: 1,
            per_page: 100,
        };

        let rows = load_rows(
            |_method, _path, _params, _payload| {
                panic!("local user browse should not hit the request layer")
            },
            &args,
            DisplayMode::GlobalAccounts,
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(
            crate::access::render::map_get_text(&rows[0], "login"),
            "alice"
        );
        assert_eq!(
            crate::access::render::map_get_text(&rows[0], "teams"),
            "ops,sre"
        );
    }

    #[test]
    fn user_detail_navigation_reaches_all_fact_rows() {
        let mut state = BrowserState::new(
            vec![Map::from_iter(vec![
                ("id".to_string(), Value::String("1".to_string())),
                ("login".to_string(), Value::String("alice".to_string())),
            ])],
            DisplayMode::GlobalAccounts,
        );

        let line_count = super::super::user_browse_dispatch::current_detail_line_count(&state);
        state.set_detail_cursor(line_count.saturating_sub(1), line_count);

        assert_eq!(line_count, 13);
        assert_eq!(state.detail_cursor, 12);
    }

    #[test]
    fn team_row_d_opens_membership_remove_confirmation_without_api() {
        let mut state = BrowserState::new(
            vec![Map::from_iter(vec![
                ("id".to_string(), Value::String("7".to_string())),
                ("login".to_string(), Value::String("alice".to_string())),
                (
                    "teamRows".to_string(),
                    Value::Array(vec![Value::Object(Map::from_iter(vec![
                        ("teamId".to_string(), Value::String("55".to_string())),
                        (
                            "teamName".to_string(),
                            Value::String("platform-ops".to_string()),
                        ),
                    ]))]),
                ),
            ])],
            DisplayMode::GlobalAccounts,
        );
        state.expand_selected();
        state.select_index(1);
        let args = UserBrowseArgs {
            common: CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            input_dir: None,
            local: false,
            run: None,
            run_id: None,
            scope: Scope::Org,
            all_orgs: false,
            current_org: false,
            query: None,
            login: None,
            email: None,
            org_role: None,
            grafana_admin: None,
            with_teams: false,
            page: 1,
            per_page: 100,
        };

        let mut request_json = |_method: Method,
                                _path: &str,
                                _params: &[(String, String)],
                                _payload: Option<&Value>| {
            panic!("membership removal preview should not call Grafana before confirmation")
        };

        handle_key(
            &mut request_json,
            &args,
            &mut state,
            &KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        )
        .unwrap();

        assert!(state.pending_member_remove);
        assert_eq!(state.status, "Previewing team membership removal.");
    }

    #[test]
    fn team_membership_remove_confirms_with_delete_and_refreshes_user_selection() {
        let mut state = BrowserState::new(
            vec![Map::from_iter(vec![
                ("id".to_string(), Value::String("7".to_string())),
                ("login".to_string(), Value::String("alice".to_string())),
                (
                    "teamRows".to_string(),
                    Value::Array(vec![Value::Object(Map::from_iter(vec![
                        ("teamId".to_string(), Value::String("55".to_string())),
                        (
                            "teamName".to_string(),
                            Value::String("platform-ops".to_string()),
                        ),
                    ]))]),
                ),
            ])],
            DisplayMode::GlobalAccounts,
        );
        state.expand_selected();
        state.select_index(1);
        state.pending_member_remove = true;

        let args = UserBrowseArgs {
            common: CommonCliArgs {
                profile: None,
                url: "http://127.0.0.1:3000".to_string(),
                api_token: None,
                username: Some("admin".to_string()),
                password: Some("admin".to_string()),
                prompt_password: false,
                prompt_token: false,
                org_id: None,
                timeout: 30,
                verify_ssl: false,
                insecure: false,
                ca_cert: None,
            },
            input_dir: None,
            local: false,
            run: None,
            run_id: None,
            scope: Scope::Org,
            all_orgs: false,
            current_org: false,
            query: None,
            login: None,
            email: None,
            org_role: None,
            grafana_admin: None,
            with_teams: false,
            page: 1,
            per_page: 100,
        };

        let mut delete_seen = false;
        let mut request_json =
            |method: Method, path: &str, _params: &[(String, String)], payload: Option<&Value>| {
                match (method.clone(), path) {
                    (Method::DELETE, "/api/teams/55/members/7") => {
                        delete_seen = true;
                        assert!(payload.is_none());
                        Ok(Some(Value::Object(Map::new())))
                    }
                    (Method::GET, "/api/org/users") => {
                        let user = Value::Object(Map::from_iter(vec![
                            ("id".to_string(), Value::String("7".to_string())),
                            ("login".to_string(), Value::String("alice".to_string())),
                            (
                                "email".to_string(),
                                Value::String("alice@example.com".to_string()),
                            ),
                            ("name".to_string(), Value::String("Alice".to_string())),
                            ("orgRole".to_string(), Value::String("Editor".to_string())),
                            ("scope".to_string(), Value::String("org".to_string())),
                        ]));
                        Ok(Some(Value::Array(vec![user])))
                    }
                    (Method::GET, "/api/users/7/teams") => {
                        if delete_seen {
                            Ok(Some(Value::Array(vec![])))
                        } else {
                            let team = Value::Object(Map::from_iter(vec![
                                ("id".to_string(), Value::String("55".to_string())),
                                (
                                    "name".to_string(),
                                    Value::String("platform-ops".to_string()),
                                ),
                            ]));
                            Ok(Some(Value::Array(vec![team])))
                        }
                    }
                    _ => panic!("unexpected request: {method:?} {path}"),
                }
            };

        handle_key(
            &mut request_json,
            &args,
            &mut state,
            &KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
        )
        .unwrap();

        assert!(delete_seen);
        assert!(!state.pending_member_remove);
        assert_eq!(state.status, "Removed membership from alice.");
        assert_eq!(state.selected_row().map(row_kind), Some("user"));
        assert_eq!(state.selected_user_id().as_deref(), Some("7"));
    }
}
