use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
#[cfg(test)]
use reqwest::Method;
use serde_json::{Map, Value};

use crate::alert::build_rule_import_payload;
use crate::common::{message, Result};
#[cfg(test)]
use crate::sync::require_json_object;
use crate::sync::{normalize_alert_managed_fields, normalize_alert_resource_identity_and_title};

use super::SyncLiveClient;

const ALERT_TEMPLATE_DETAIL_FETCH_CONCURRENCY: usize = 8;

pub(super) fn append_alert_resource_specs_from_client(
    client: &SyncLiveClient<'_>,
    specs: &mut Vec<Value>,
) -> Result<()> {
    for rule in client.list_alert_rules()? {
        append_alert_rule_spec(&rule, specs)?;
    }

    for contact_point in client.list_contact_points()? {
        specs.push(build_live_alert_resource_spec(
            "alert-contact-point",
            contact_point,
        )?);
    }

    for mute_timing in client.list_mute_timings()? {
        specs.push(build_live_alert_resource_spec(
            "alert-mute-timing",
            mute_timing,
        )?);
    }

    specs.push(build_live_alert_resource_spec(
        "alert-policy",
        client.get_notification_policies()?,
    )?);

    append_alert_template_specs_from_summaries(specs, client.list_templates()?, |name| {
        client.get_template(name)
    })?;

    Ok(())
}

#[cfg(test)]
pub(super) fn append_alert_resource_specs_with_request<F>(
    request_json: &mut F,
    specs: &mut Vec<Value>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    match request_json(Method::GET, "/api/v1/provisioning/alert-rules", &[], None)? {
        Some(Value::Array(rules)) => {
            for rule in rules {
                let object = require_json_object(&rule, "Grafana alert-rule payload")?;
                append_alert_rule_spec(object, specs)?;
            }
        }
        Some(_) => return Err(message("Unexpected alert-rule list response from Grafana.")),
        None => {}
    }

    match request_json(
        Method::GET,
        "/api/v1/provisioning/contact-points",
        &[],
        None,
    )? {
        Some(Value::Array(contact_points)) => {
            for contact_point in contact_points {
                let object = require_json_object(&contact_point, "Grafana contact-point payload")?;
                specs.push(build_live_alert_resource_spec(
                    "alert-contact-point",
                    object.clone(),
                )?);
            }
        }
        Some(_) => {
            return Err(message(
                "Unexpected contact-point list response from Grafana.",
            ))
        }
        None => {}
    }

    match request_json(Method::GET, "/api/v1/provisioning/mute-timings", &[], None)? {
        Some(Value::Array(mute_timings)) => {
            for mute_timing in mute_timings {
                let object = require_json_object(&mute_timing, "Grafana mute-timing payload")?;
                specs.push(build_live_alert_resource_spec(
                    "alert-mute-timing",
                    object.clone(),
                )?);
            }
        }
        Some(_) => {
            return Err(message(
                "Unexpected mute-timing list response from Grafana.",
            ))
        }
        None => {}
    }

    match request_json(Method::GET, "/api/v1/provisioning/policies", &[], None)? {
        Some(Value::Object(policies)) => {
            specs.push(build_live_alert_resource_spec(
                "alert-policy",
                policies.clone(),
            )?);
        }
        Some(_) => {
            return Err(message(
                "Unexpected notification policy response from Grafana.",
            ))
        }
        None => {}
    }

    match request_json(Method::GET, "/api/v1/provisioning/templates", &[], None)? {
        Some(Value::Array(templates)) => {
            for template in templates {
                let object = require_json_object(&template, "Grafana template summary payload")?;
                let name = template_name(object)?;
                let template_payload = match request_json(
                    Method::GET,
                    &format!("/api/v1/provisioning/templates/{name}"),
                    &[],
                    None,
                )? {
                    Some(Value::Object(template_object)) => template_object,
                    Some(_) => return Err(message("Unexpected template payload from Grafana.")),
                    None => continue,
                };
                specs.push(build_live_alert_resource_spec(
                    "alert-template",
                    template_payload,
                )?);
            }
        }
        Some(Value::Null) => {}
        Some(_) => return Err(message("Unexpected template list response from Grafana.")),
        None => {}
    }

    Ok(())
}

fn append_alert_rule_spec(rule: &Map<String, Value>, specs: &mut Vec<Value>) -> Result<()> {
    let body = build_rule_import_payload(rule)?;
    let uid = body
        .get("uid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message("Live alert rule payload is missing uid."))?;
    let title = body
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(uid);
    specs.push(serde_json::json!({
        "kind": "alert",
        "uid": uid,
        "title": title,
        "body": body,
    }));
    Ok(())
}

fn build_live_alert_resource_spec(sync_kind: &str, body: Map<String, Value>) -> Result<Value> {
    let (identity, title) = normalize_alert_resource_identity_and_title(sync_kind, &body)?;
    Ok(serde_json::json!({
        "kind": sync_kind,
        "uid": if sync_kind == "alert-contact-point" { identity.clone() } else { String::new() },
        "name": if matches!(sync_kind, "alert-mute-timing" | "alert-template") { identity.clone() } else { String::new() },
        "title": title,
        "managedFields": normalize_alert_managed_fields(&body),
        "body": body,
    }))
}

fn template_name(template: &Map<String, Value>) -> Result<&str> {
    template
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message("Live template payload is missing name."))
}

fn append_alert_template_specs_from_summaries<F>(
    specs: &mut Vec<Value>,
    templates: Vec<Map<String, Value>>,
    fetch_template: F,
) -> Result<()>
where
    F: Fn(&str) -> Result<Map<String, Value>> + Sync,
{
    let names = templates
        .iter()
        .map(|template| template_name(template).map(str::to_string))
        .collect::<Result<Vec<_>>>()?;
    if names.is_empty() {
        return Ok(());
    }

    let pool = ThreadPoolBuilder::new()
        .num_threads(ALERT_TEMPLATE_DETAIL_FETCH_CONCURRENCY)
        .build()
        .map_err(|error| {
            message(format!(
                "Failed to build alert template detail read worker pool: {error}"
            ))
        })?;
    let reads = pool.install(|| {
        names
            .par_iter()
            .map(|name| (name.clone(), fetch_template(name)))
            .collect::<Vec<_>>()
    });

    let failures = reads
        .iter()
        .filter_map(|(name, result)| {
            result
                .as_ref()
                .err()
                .map(|error| format!("name={name}: {error}"))
        })
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        return Err(message(format!(
            "Failed to fetch {} alert template detail(s) after /api/v1/provisioning/templates: {}",
            failures.len(),
            failures.join("; ")
        )));
    }

    let mut template_specs = Vec::new();
    for (_, result) in reads {
        template_specs.push(build_live_alert_resource_spec("alert-template", result?)?);
    }
    specs.extend(template_specs);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template_summary(name: &str) -> Map<String, Value> {
        let mut object = Map::new();
        object.insert("name".to_string(), Value::String(name.to_string()));
        object
    }

    fn template_payload(name: &str) -> Map<String, Value> {
        let mut object = Map::new();
        object.insert("name".to_string(), Value::String(name.to_string()));
        object.insert(
            "template".to_string(),
            Value::String(format!("{{{{ define \"{name}\" }}}}ok{{{{ end }}}}")),
        );
        object
    }

    #[test]
    fn append_alert_template_specs_from_summaries_preserves_template_list_order() {
        let mut specs = Vec::new();

        append_alert_template_specs_from_summaries(
            &mut specs,
            vec![
                template_summary("template-b"),
                template_summary("template-a"),
                template_summary("template-c"),
            ],
            |name| Ok(template_payload(name)),
        )
        .unwrap();

        let names = specs
            .iter()
            .map(|spec| spec["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["template-b", "template-a", "template-c"]);
    }

    #[test]
    fn append_alert_template_specs_from_summaries_aggregates_detail_failures() {
        let mut specs = vec![serde_json::json!({"kind": "alert-policy"})];

        let error = append_alert_template_specs_from_summaries(
            &mut specs,
            vec![
                template_summary("template-a"),
                template_summary("template-b"),
                template_summary("template-c"),
            ],
            |name| {
                if name == "template-b" || name == "template-c" {
                    Err(message(format!("boom {name}")))
                } else {
                    Ok(template_payload(name))
                }
            },
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains(
            "Failed to fetch 2 alert template detail(s) after /api/v1/provisioning/templates"
        ));
        assert!(error.contains("name=template-b: boom template-b"));
        assert!(error.contains("name=template-c: boom template-c"));
        assert_eq!(specs, vec![serde_json::json!({"kind": "alert-policy"})]);
    }
}
