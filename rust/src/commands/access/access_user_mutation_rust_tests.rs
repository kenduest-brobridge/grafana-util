use super::super::*;

#[test]
fn user_add_with_request_requires_basic_auth_and_updates_role() {
    let args = UserAddArgs {
        common: make_basic_common(),
        login: "alice".to_string(),
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
        new_user_password: Some("pw".to_string()),
        new_user_password_file: None,
        prompt_user_password: false,
        org_role: Some("Editor".to_string()),
        grafana_admin: Some(true),
        json: true,
    };
    let mut calls = Vec::new();
    let result = add_user_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match path {
                "/api/admin/users" => Ok(Some(json!({"id": 9}))),
                "/api/org/users/9" => Ok(Some(json!({"message": "ok"}))),
                "/api/admin/users/9/permissions" => Ok(Some(json!({"message": "ok"}))),
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/admin/users"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/org/users/9"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/admin/users/9/permissions"));
}

#[test]
fn user_add_with_request_reads_password_file() {
    let temp_dir = tempdir().unwrap();
    let password_path = temp_dir.path().join("user-password.txt");
    fs::write(&password_path, "pw-from-file\n").unwrap();
    let args = UserAddArgs {
        common: make_basic_common(),
        login: "alice".to_string(),
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
        new_user_password: None,
        new_user_password_file: Some(password_path.clone()),
        prompt_user_password: false,
        org_role: None,
        grafana_admin: None,
        json: false,
    };
    let mut captured_password = None;
    let result = add_user_with_request(
        |method, path, _params, payload| {
            if method == Method::POST && path == "/api/admin/users" {
                captured_password = payload
                    .and_then(|value| value.get("password"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string);
                return Ok(Some(json!({"id": 9})));
            }
            panic!("unexpected request");
        },
        &args,
    );

    assert!(result.is_ok());
    assert_eq!(captured_password.as_deref(), Some("pw-from-file"));
}

#[test]
fn user_modify_with_request_updates_profile_and_password() {
    let args = UserModifyArgs {
        common: make_basic_common(),
        user_id: Some("9".to_string()),
        login: None,
        email: None,
        set_login: Some("alice2".to_string()),
        set_email: None,
        set_name: Some("Alice Two".to_string()),
        set_password: Some("newpw".to_string()),
        set_password_file: None,
        prompt_set_password: false,
        set_org_role: None,
        set_grafana_admin: None,
        json: true,
    };
    let mut calls = Vec::new();
    let result = modify_user_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            match path {
                "/api/users/9" if method == Method::GET => Ok(Some(
                    json!({"id": 9, "login": "alice", "email": "alice@example.com", "name": "Alice"}),
                )),
                "/api/users/9" if method == Method::PUT => Ok(Some(json!({"message": "ok"}))),
                "/api/admin/users/9/password" => Ok(Some(json!({"message": "ok"}))),
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls
        .iter()
        .any(|(method, path, _, _)| method == "PUT" && path == "/api/users/9"));
    assert!(calls
        .iter()
        .any(|(_, path, _, _)| path == "/api/admin/users/9/password"));
}

#[test]
fn user_modify_with_request_reads_set_password_file() {
    let temp_dir = tempdir().unwrap();
    let password_path = temp_dir.path().join("replacement-password.txt");
    fs::write(&password_path, "newpw-from-file\n").unwrap();
    let args = UserModifyArgs {
        common: make_basic_common(),
        user_id: Some("9".to_string()),
        login: None,
        email: None,
        set_login: None,
        set_email: None,
        set_name: None,
        set_password: None,
        set_password_file: Some(password_path.clone()),
        prompt_set_password: false,
        set_org_role: None,
        set_grafana_admin: None,
        json: false,
    };
    let mut captured_password = None;
    let result = modify_user_with_request(
        |method, path, _params, payload| match path {
            "/api/users/9" if method == Method::GET => Ok(Some(
                json!({"id": 9, "login": "alice", "email": "alice@example.com", "name": "Alice"}),
            )),
            "/api/admin/users/9/password" if method == Method::PUT => {
                captured_password = payload
                    .and_then(|value| value.get("password"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string);
                Ok(Some(json!({"message": "ok"})))
            }
            _ => panic!("unexpected request"),
        },
        &args,
    );

    assert!(result.is_ok());
    assert_eq!(captured_password.as_deref(), Some("newpw-from-file"));
}

#[test]
fn user_delete_with_request_requires_yes_and_deletes() {
    let args = UserDeleteArgs {
        common: make_basic_common(),
        user_id: Some("9".to_string()),
        login: None,
        email: None,
        scope: Some(Scope::Global),
        prompt: false,
        yes: true,
        json: true,
    };
    let mut calls = Vec::new();
    let result = delete_user_with_request(
        |method, path, params, _payload| {
            calls.push((method.to_string(), path.to_string(), params.to_vec()));
            match path {
                "/api/users/9" if method == Method::GET => {
                    Ok(Some(json!({"id": 9, "login": "alice"})))
                }
                "/api/admin/users/9" if method == Method::DELETE => {
                    Ok(Some(json!({"message": "deleted"})))
                }
                _ => panic!("unexpected path {path}"),
            }
        },
        &args,
    );

    assert!(result.is_ok());
    assert!(calls
        .iter()
        .any(|(method, path, _)| method == "DELETE" && path == "/api/admin/users/9"));
}

#[test]
fn run_access_cli_with_request_routes_user_list() {
    let args = parse_cli_from([
        "grafana-util access",
        "user",
        "list",
        "--url",
        "https://grafana.example.com",
        "--json",
        "--token",
        "abc",
    ]);
    let result = run_access_cli_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org/users" => Ok(Some(json!([]))),
            _ => panic!("unexpected path {path}"),
        },
        &args,
    );
    assert!(result.is_ok());
}
