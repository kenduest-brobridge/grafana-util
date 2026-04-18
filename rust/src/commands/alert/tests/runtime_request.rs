use crate::common::api_response;
use crate::grafana_api::alert_live::request_optional_object_with_request;
use reqwest::Method;

#[test]
fn request_optional_object_with_request_treats_http_404_as_missing() {
    let result = request_optional_object_with_request(
        |_method, path, _params, _payload| {
            Err(api_response(
                404,
                format!("http://127.0.0.1:3000{path}"),
                "",
            ))
        },
        Method::GET,
        "/api/v1/provisioning/alert-rules/missing-rule",
        None,
    )
    .unwrap();

    assert!(result.is_none());
}
