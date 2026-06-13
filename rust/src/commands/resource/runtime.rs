use reqwest::Method;
use serde_json::{Map, Value};

use super::catalog::ResourceSelector;
use super::cli_defs::{ResourceApiMode, ResourceCliArgs, ResourceCommand, ResourceKind};
use super::render::{render_describe, render_get, render_kind_catalog, render_list};
use crate::common::{set_json_color_choice, Result};
use crate::dashboard::{CommonCliArgs, DEFAULT_TIMEOUT, DEFAULT_URL};
use crate::grafana_api::{
    expect_object, expect_object_list, AuthInputs, GrafanaApiClient, GrafanaConnection,
};
use crate::profile_config::ConnectionMergeInput;

pub(crate) fn build_client(common: &CommonCliArgs) -> Result<GrafanaApiClient> {
    let connection = GrafanaConnection::resolve(
        common.profile.as_deref(),
        ConnectionMergeInput {
            url: &common.url,
            url_default: DEFAULT_URL,
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            org_id: None,
            timeout: common.timeout,
            timeout_default: DEFAULT_TIMEOUT,
            verify_ssl: common.verify_ssl,
            insecure: false,
            ca_cert: None,
        },
        AuthInputs {
            api_token: common.api_token.as_deref(),
            username: common.username.as_deref(),
            password: common.password.as_deref(),
            prompt_password: common.prompt_password,
            prompt_token: common.prompt_token,
        },
        false,
    )?;
    GrafanaApiClient::from_connection(connection)
}

pub(crate) fn list_resource_items(
    client: &GrafanaApiClient,
    kind: ResourceKind,
    api_mode: ResourceApiMode,
) -> Result<Vec<Map<String, Value>>> {
    let items = match kind {
        ResourceKind::Dashboards => resolve_with_api_mode(
            api_mode,
            &|| client.dashboard().list_dashboard_summaries(500),
            None,
        ),
        ResourceKind::Folders => resolve_with_api_mode(
            api_mode,
            &|| {
                expect_object_list(
                    client
                        .dashboard()
                        .request_json(Method::GET, "/api/folders", &[], None)?,
                    "Unexpected folder list response from Grafana.",
                )
            },
            None,
        ),
        ResourceKind::Datasources => {
            resolve_with_api_mode(api_mode, &|| client.datasource().list_datasources(), None)
        }
        ResourceKind::AlertRules => {
            resolve_with_api_mode(api_mode, &|| client.alerting().list_alert_rules(), None)
        }
        ResourceKind::Orgs => {
            resolve_with_api_mode(api_mode, &|| client.access().list_orgs(), None)
        }
    };
    items
}

pub(crate) fn get_resource_item(
    client: &GrafanaApiClient,
    selector: &ResourceSelector,
    api_mode: ResourceApiMode,
) -> Result<Value> {
    match selector.kind {
        ResourceKind::Dashboards => resolve_with_api_mode(
            api_mode,
            &|| client.dashboard().fetch_dashboard(&selector.identity),
            None,
        ),
        ResourceKind::Folders => {
            let object = resolve_with_api_mode(
                api_mode,
                &|| {
                    client.dashboard().request_json(
                        Method::GET,
                        &format!("/api/folders/{}", selector.identity),
                        &[],
                        None,
                    )
                },
                None,
            )?;
            Ok(Value::Object(expect_object(
                object,
                "Unexpected folder payload from Grafana.",
            )?))
        }
        ResourceKind::Datasources => {
            let object = resolve_with_api_mode(
                api_mode,
                &|| {
                    client.datasource().request_json(
                        Method::GET,
                        &format!("/api/datasources/uid/{}", selector.identity),
                        &[],
                        None,
                    )
                },
                Some(&|| {
                    client.datasource().request_json(
                        Method::GET,
                        &format!("/api/datasources/{}", selector.identity),
                        &[],
                        None,
                    )
                }),
            )?;
            Ok(Value::Object(expect_object(
                object,
                "Unexpected datasource payload from Grafana.",
            )?))
        }
        ResourceKind::AlertRules => Ok(Value::Object(resolve_with_api_mode(
            api_mode,
            &|| client.alerting().get_alert_rule(&selector.identity),
            None,
        )?)),
        ResourceKind::Orgs => {
            let object = resolve_with_api_mode(
                api_mode,
                &|| {
                    client.access().request_json(
                        Method::GET,
                        &format!("/api/orgs/{}", selector.identity),
                        &[],
                        None,
                    )
                },
                None,
            )?;
            Ok(Value::Object(expect_object(
                object,
                "Unexpected org payload from Grafana.",
            )?))
        }
    }
}

fn resolve_with_api_mode<T>(
    mode: ResourceApiMode,
    modern: &dyn Fn() -> Result<T>,
    legacy: Option<&dyn Fn() -> Result<T>>,
) -> Result<T> {
    match mode {
        ResourceApiMode::Auto => match modern() {
            Ok(value) => Ok(value),
            Err(error) if error.status_code() == Some(404) => {
                if let Some(legacy_attempt) = legacy {
                    legacy_attempt()
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        },
        ResourceApiMode::Legacy => {
            if let Some(legacy_attempt) = legacy {
                legacy_attempt()
            } else {
                modern()
            }
        }
    }
}

pub fn run_resource_cli(args: ResourceCliArgs) -> Result<()> {
    // Resource is a pure read-only surface; dispatch only routes to renderers,
    // with output shape determined by each command variant.
    set_json_color_choice(args.color);
    match args.command {
        ResourceCommand::Kinds(inner) => render_kind_catalog(&inner),
        ResourceCommand::Describe(inner) => render_describe(&inner),
        ResourceCommand::List(inner) => render_list(&inner),
        ResourceCommand::Get(inner) => render_get(&inner),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::super::catalog::ResourceSelector;
    use super::super::cli_defs::ResourceApiMode;
    use super::super::cli_defs::ResourceKind;
    use crate::common::{api_response, message, Result};
    use crate::grafana_api::{GrafanaApiClient, GrafanaConnection};

    type SequenceServer = (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>);

    fn http_response(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn spawn_sequence_server(responses: Vec<String>) -> Option<SequenceServer> {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == ErrorKind::PermissionDenied => return None,
            Err(error) => panic!("failed to bind test listener: {error}"),
        };
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();

                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..bytes_read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                let request_line = String::from_utf8_lossy(&request)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                requests_thread.lock().unwrap().push(request_line);

                stream.write_all(response.as_bytes()).unwrap();
                let _ = stream.flush();
            }
        });
        Some((format!("http://{address}"), requests, handle))
    }

    fn build_test_api(base_url: String) -> GrafanaApiClient {
        GrafanaApiClient::from_connection(GrafanaConnection::new(
            base_url,
            vec![("Authorization".to_string(), "Bearer token".to_string())],
            5,
            false,
            None,
            "token".to_string(),
        ))
        .unwrap()
    }

    fn datasource_selector(identity: &str) -> ResourceSelector {
        ResourceSelector {
            kind: ResourceKind::Datasources,
            identity: identity.to_string(),
        }
    }

    #[test]
    fn resolve_with_api_mode_auto_uses_legacy_when_modern_404s() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Auto,
            &|| {
                Err(api_response(
                    404,
                    "http://grafana.local/api/datasources/uid/1",
                    "not found",
                ))
            },
            Some(&|| Ok(42)),
        );
        assert_eq!(value.unwrap(), 42);
    }

    #[test]
    fn resolve_with_api_mode_auto_preserves_non_404_legacy_error() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Auto,
            &|| {
                Err(api_response(
                    403,
                    "http://grafana.local/api/datasources/uid/1",
                    "forbidden",
                ))
            },
            Some(&|| unreachable!("legacy path should not be called when modern returns non-404")),
        );
        let error = value.unwrap_err();
        assert_eq!(error.status_code(), Some(403));
    }

    #[test]
    fn resolve_with_api_mode_legacy_does_not_use_modern_path() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Legacy,
            &|| unreachable!("modern path should not be called in legacy mode"),
            Some(&|| Ok(7)),
        );
        assert_eq!(value.unwrap(), 7);
    }

    #[test]
    fn resolve_with_api_mode_legacy_falls_back_to_modern_when_legacy_not_available() {
        let value: Result<i32> =
            super::resolve_with_api_mode(ResourceApiMode::Legacy, &|| Ok(99), None);
        assert_eq!(value.unwrap(), 99);
    }

    #[test]
    fn resolve_with_api_mode_legacy_returns_error_when_legacy_missing() {
        let value: Result<i32> = super::resolve_with_api_mode(
            ResourceApiMode::Legacy,
            &|| Err(message("modern-only")),
            None,
        );
        assert_eq!(
            value.err().map(|error| error.to_string()),
            Some("modern-only".to_string())
        );
    }

    #[test]
    fn get_datasource_auto_falls_back_from_uid_path_to_legacy_path_on_404() {
        let Some((base_url, requests, handle)) = spawn_sequence_server(vec![
            http_response("404 Not Found", r#"{"message":"not found"}"#),
            http_response(
                "200 OK",
                r#"{"id":10,"uid":"prom-main","name":"Prometheus"}"#,
            ),
        ]) else {
            return;
        };
        let client = build_test_api(base_url);
        let value =
            super::get_resource_item(&client, &datasource_selector("10"), ResourceApiMode::Auto)
                .unwrap();

        assert_eq!(value["name"], "Prometheus");
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests[0], "GET /api/datasources/uid/10 HTTP/1.1");
        assert_eq!(requests[1], "GET /api/datasources/10 HTTP/1.1");
    }

    #[test]
    fn get_datasource_legacy_uses_legacy_path_without_uid_probe() {
        let Some((base_url, requests, handle)) = spawn_sequence_server(vec![http_response(
            "200 OK",
            r#"{"id":10,"uid":"prom-main","name":"Prometheus"}"#,
        )]) else {
            return;
        };
        let client = build_test_api(base_url);
        let value =
            super::get_resource_item(&client, &datasource_selector("10"), ResourceApiMode::Legacy)
                .unwrap();

        assert_eq!(value["uid"], "prom-main");
        handle.join().unwrap();
        let requests = requests.lock().unwrap();
        assert_eq!(requests.as_slice(), ["GET /api/datasources/10 HTTP/1.1"]);
    }
}
