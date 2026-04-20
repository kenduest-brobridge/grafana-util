use reqwest::Method;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

use crate::common::{message, string_field, value_as_object, Result};
use crate::grafana_api::DashboardResourceClient;
use crate::sync::preflight::build_sync_preflight_document;

use super::super::list::collect_dashboard_source_metadata;
use super::super::{build_datasource_catalog, collect_datasource_refs, DEFAULT_DASHBOARD_TITLE};
use super::super::{discover_dashboard_files, FOLDER_INVENTORY_FILENAME};

mod dependency_schema {
    pub(super) mod availability {
        pub(crate) const DATASOURCE_UIDS: &str = "datasourceUids";
        pub(crate) const DATASOURCE_NAMES: &str = "datasourceNames";
        pub(crate) const PLUGIN_IDS: &str = "pluginIds";
    }

    pub(super) mod spec {
        pub(crate) const KIND: &str = "kind";
        pub(crate) const UID: &str = "uid";
        pub(crate) const TITLE: &str = "title";
        pub(crate) const BODY: &str = "body";
        pub(crate) const SOURCE_PATH: &str = "sourcePath";
    }

    pub(super) mod summary {
        pub(crate) const ROOT: &str = "summary";
        pub(crate) const BLOCKING_COUNT: &str = "blockingCount";
    }
}

fn collect_dashboard_panel_types(panels: &[Value], panel_types: &mut BTreeSet<String>) {
    for panel in panels {
        let Some(panel_object) = panel.as_object() else {
            continue;
        };
        let panel_type = string_field(panel_object, "type", "");
        if !panel_type.is_empty() {
            panel_types.insert(panel_type);
        }
        if let Some(nested) = panel_object.get("panels").and_then(Value::as_array) {
            collect_dashboard_panel_types(nested, panel_types);
        }
    }
}

fn dashboard_import_dependency_availability_requirements(input_dir: &Path) -> Result<(bool, bool)> {
    let mut dashboard_files = discover_dashboard_files(input_dir)?;
    dashboard_files.retain(|path| {
        path.file_name().and_then(|name| name.to_str()) != Some(FOLDER_INVENTORY_FILENAME)
    });
    let mut needs_datasource_availability = false;
    let mut needs_plugin_availability = false;
    for dashboard_file in dashboard_files {
        let document = super::super::load_json_file(&dashboard_file)?;
        let document_object =
            value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = super::super::extract_dashboard_object(document_object)?;
        let mut refs = Vec::new();
        collect_datasource_refs(&Value::Object(dashboard.clone()), &mut refs);
        if refs
            .iter()
            .any(|reference| !super::super::is_builtin_datasource_ref(reference))
        {
            needs_datasource_availability = true;
        }
        if let Some(panels) = dashboard.get("panels").and_then(Value::as_array) {
            let mut panel_types = BTreeSet::new();
            collect_dashboard_panel_types(panels, &mut panel_types);
            if !panel_types.is_empty() {
                needs_plugin_availability = true;
            }
        }
        if needs_datasource_availability && needs_plugin_availability {
            break;
        }
    }
    Ok((needs_datasource_availability, needs_plugin_availability))
}

fn sorted_strings_value(values: BTreeSet<String>) -> Value {
    Value::Array(values.into_iter().map(Value::String).collect::<Vec<_>>())
}

fn collect_plugin_ids(plugins: &[Value]) -> BTreeSet<String> {
    plugins
        .iter()
        .filter_map(Value::as_object)
        .filter_map(|plugin| plugin.get("id").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<String>>()
}

fn insert_plugin_ids(availability: &mut Map<String, Value>, plugins: &[Value]) {
    availability.insert(
        dependency_schema::availability::PLUGIN_IDS.to_string(),
        sorted_strings_value(collect_plugin_ids(plugins)),
    );
}

fn build_dashboard_import_availability_from_datasources(
    datasources: &[Map<String, Value>],
) -> Map<String, Value> {
    let mut availability = Map::new();
    let mut datasource_uids = BTreeSet::new();
    let mut datasource_names = BTreeSet::new();
    for datasource in datasources {
        if let Some(uid) = datasource
            .get("uid")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            datasource_uids.insert(uid.to_string());
        }
        if let Some(name) = datasource
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            datasource_names.insert(name.to_string());
        }
    }
    availability.insert(
        dependency_schema::availability::DATASOURCE_UIDS.to_string(),
        sorted_strings_value(datasource_uids),
    );
    availability.insert(
        dependency_schema::availability::DATASOURCE_NAMES.to_string(),
        sorted_strings_value(datasource_names),
    );
    availability.insert(
        dependency_schema::availability::PLUGIN_IDS.to_string(),
        Value::Array(Vec::new()),
    );
    availability
}

fn build_dashboard_import_availability_with_request<F>(
    mut request_json: F,
    datasources: &[Map<String, Value>],
    fetch_plugins: bool,
) -> Result<Map<String, Value>>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let mut availability = build_dashboard_import_availability_from_datasources(datasources);
    if !fetch_plugins {
        return Ok(availability);
    }
    match request_json(Method::GET, "/api/plugins", &[], None)? {
        Some(Value::Array(plugins)) => {
            insert_plugin_ids(&mut availability, &plugins);
        }
        Some(_) => return Err(message("Unexpected plugin list response from Grafana.")),
        None => {}
    }

    Ok(availability)
}

fn build_dashboard_import_availability_with_client(
    client: &DashboardResourceClient<'_>,
    datasources: &[Map<String, Value>],
    fetch_plugins: bool,
) -> Result<Map<String, Value>> {
    let mut availability = build_dashboard_import_availability_from_datasources(datasources);
    if !fetch_plugins {
        return Ok(availability);
    }
    match client.request_json(Method::GET, "/api/plugins", &[], None)? {
        Some(Value::Array(plugins)) => {
            insert_plugin_ids(&mut availability, &plugins);
        }
        Some(_) => return Err(message("Unexpected plugin list response from Grafana.")),
        None => {}
    }

    Ok(availability)
}

fn build_dashboard_import_dependency_specs(
    input_dir: &Path,
    datasource_catalog: &super::super::prompt::DatasourceCatalog,
    strict_schema: bool,
    target_schema_version: Option<i64>,
) -> Result<Vec<Value>> {
    let mut dashboard_files = discover_dashboard_files(input_dir)?;
    dashboard_files.retain(|path| {
        path.file_name().and_then(|name| name.to_str()) != Some(FOLDER_INVENTORY_FILENAME)
    });
    let mut desired_specs = Vec::new();
    for dashboard_file in dashboard_files {
        let document = super::super::load_json_file(&dashboard_file)?;
        super::super::validate::validate_dashboard_import_document(
            &document,
            &dashboard_file,
            strict_schema,
            target_schema_version,
        )?;
        let document_object =
            value_as_object(&document, "Dashboard payload must be a JSON object.")?;
        let dashboard = super::super::extract_dashboard_object(document_object)?;
        let uid = string_field(dashboard, "uid", "");
        let title = string_field(dashboard, "title", DEFAULT_DASHBOARD_TITLE);
        let (datasource_names, datasource_uids) =
            collect_dashboard_source_metadata(&document, datasource_catalog)?;
        let mut panel_types = BTreeSet::new();
        if let Some(panels) = dashboard.get("panels").and_then(Value::as_array) {
            collect_dashboard_panel_types(panels, &mut panel_types);
        }
        let mut body = Map::new();
        body.insert(
            dependency_schema::availability::DATASOURCE_NAMES.to_string(),
            serde_json::json!(datasource_names),
        );
        body.insert(
            dependency_schema::availability::DATASOURCE_UIDS.to_string(),
            serde_json::json!(datasource_uids),
        );
        body.insert(
            dependency_schema::availability::PLUGIN_IDS.to_string(),
            serde_json::json!(panel_types.into_iter().collect::<Vec<String>>()),
        );

        let mut spec = Map::new();
        spec.insert(
            dependency_schema::spec::KIND.to_string(),
            Value::String("dashboard".to_string()),
        );
        spec.insert(
            dependency_schema::spec::UID.to_string(),
            Value::String(uid.to_string()),
        );
        spec.insert(
            dependency_schema::spec::TITLE.to_string(),
            Value::String(title.to_string()),
        );
        spec.insert(
            dependency_schema::spec::BODY.to_string(),
            Value::Object(body),
        );
        spec.insert(
            dependency_schema::spec::SOURCE_PATH.to_string(),
            Value::String(dashboard_file.display().to_string()),
        );
        desired_specs.push(Value::Object(spec));
    }
    Ok(desired_specs)
}

fn preflight_blocking_count(document: &Value) -> i64 {
    document
        .get(dependency_schema::summary::ROOT)
        .and_then(Value::as_object)
        .and_then(|summary| summary.get(dependency_schema::summary::BLOCKING_COUNT))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

pub(crate) fn validate_dashboard_import_dependencies_with_request<F>(
    mut request_json: F,
    input_dir: &Path,
    strict_schema: bool,
    target_schema_version: Option<i64>,
) -> Result<()>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let (needs_datasource_availability, needs_plugin_availability) =
        dashboard_import_dependency_availability_requirements(input_dir)?;
    let datasources = if needs_datasource_availability {
        crate::dashboard::list_datasources_with_request(&mut request_json)?
    } else {
        Vec::new()
    };
    let datasource_catalog = build_datasource_catalog(&datasources);
    let desired_specs = build_dashboard_import_dependency_specs(
        input_dir,
        &datasource_catalog,
        strict_schema,
        target_schema_version,
    )?;
    let availability = build_dashboard_import_availability_with_request(
        &mut request_json,
        &datasources,
        needs_plugin_availability,
    )?;
    let document =
        build_sync_preflight_document(&desired_specs, Some(&Value::Object(availability)))?;
    let blocking = preflight_blocking_count(&document);
    if blocking > 0 {
        return Err(message(format!(
            "Refusing dashboard import because preflight reports {blocking} blocking checks."
        )));
    }
    Ok(())
}

pub(crate) fn validate_dashboard_import_dependencies_with_client(
    client: &DashboardResourceClient<'_>,
    input_dir: &Path,
    strict_schema: bool,
    target_schema_version: Option<i64>,
) -> Result<()> {
    let (needs_datasource_availability, needs_plugin_availability) =
        dashboard_import_dependency_availability_requirements(input_dir)?;
    let datasources = if needs_datasource_availability {
        client.list_datasources()?
    } else {
        Vec::new()
    };
    let datasource_catalog = build_datasource_catalog(&datasources);
    let desired_specs = build_dashboard_import_dependency_specs(
        input_dir,
        &datasource_catalog,
        strict_schema,
        target_schema_version,
    )?;
    let availability = build_dashboard_import_availability_with_client(
        client,
        &datasources,
        needs_plugin_availability,
    )?;
    let document =
        build_sync_preflight_document(&desired_specs, Some(&Value::Object(availability)))?;
    let blocking = preflight_blocking_count(&document);
    if blocking > 0 {
        return Err(message(format!(
            "Refusing dashboard import because preflight reports {blocking} blocking checks."
        )));
    }
    Ok(())
}
