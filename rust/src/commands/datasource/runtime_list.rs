//! Datasource runtime list, type catalog, and browse handlers.

use serde_json::Value;

use crate::common::{message, render_json_value, string_field, Result};
use crate::dashboard::{
    build_api_client, build_auth_context, build_http_client, build_http_client_for_org_from_api,
    SimpleOutputFormat,
};
use crate::datasource_catalog::{
    render_supported_datasource_catalog_csv, render_supported_datasource_catalog_json,
    render_supported_datasource_catalog_table, render_supported_datasource_catalog_text,
    render_supported_datasource_catalog_yaml,
};
use crate::grafana_api::DatasourceResourceClient;
use crate::tabular_output::render_yaml;

pub(super) fn run_datasource_types(args: super::DatasourceTypesArgs) -> Result<()> {
    match args.output_format {
        SimpleOutputFormat::Text => {
            for line in render_supported_datasource_catalog_text() {
                println!("{line}");
            }
        }
        SimpleOutputFormat::Table => {
            for line in render_supported_datasource_catalog_table() {
                println!("{line}");
            }
        }
        SimpleOutputFormat::Csv => {
            for line in render_supported_datasource_catalog_csv() {
                println!("{line}");
            }
        }
        SimpleOutputFormat::Json => {
            print!(
                "{}",
                render_json_value(&render_supported_datasource_catalog_json())?
            );
        }
        SimpleOutputFormat::Yaml => {
            print!("{}", render_supported_datasource_catalog_yaml()?);
        }
    }
    Ok(())
}

pub(super) fn run_datasource_list(mut args: super::DatasourceListArgs) -> Result<()> {
    if args.local && args.input_dir.is_none() {
        args.input_dir = Some(super::datasource_runtime_artifacts::resolve_datasource_artifact_input_dir(
            args.common.profile.as_deref(),
            args.run,
            args.run_id.as_deref(),
        )?);
    }
    if args.input_dir.is_some() {
        return super::run_local_datasource_list(&args);
    }
    if args.interactive {
        return Err(message(
            "Datasource list --interactive requires --input-dir. Use datasource browse for live interactive review.",
        ));
    }
    let datasources = if args.all_orgs {
        let context = build_auth_context(&args.common)?;
        if context.auth_mode != "basic" {
            return Err(message(
                "Datasource list with --all-orgs requires Basic auth (--basic-user / --basic-password).",
            ));
        }
        let admin_api = build_api_client(&args.common)?;
        let admin_client = admin_api.http_client();
        let admin_datasource = DatasourceResourceClient::new(admin_client);
        let mut rows = Vec::new();
        for org in admin_datasource.list_orgs()? {
            let org_id = org
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| message("Grafana org list entry is missing numeric id."))?;
            let org_client = build_http_client_for_org_from_api(&admin_api, org_id)?;
            rows.extend(super::build_list_records(&org_client)?);
        }
        rows.sort_by(|left, right| {
            let left_org_id = string_field(left, "orgId", "");
            let right_org_id = string_field(right, "orgId", "");
            left_org_id
                .cmp(&right_org_id)
                .then_with(|| string_field(left, "name", "").cmp(&string_field(right, "name", "")))
                .then_with(|| string_field(left, "uid", "").cmp(&string_field(right, "uid", "")))
        });
        rows
    } else if args.org_id.is_some() {
        let client = super::resolve_target_client(&args.common, args.org_id)?;
        super::build_list_records(&client)?
    } else {
        let client = build_http_client(&args.common)?;
        DatasourceResourceClient::new(&client).list_datasources()?
    };
    if args.json {
        print!(
            "{}",
            render_json_value(&super::render_data_source_json(
                &datasources,
                (!args.output_columns.is_empty()).then_some(args.output_columns.as_slice()),
            ))?
        );
    } else if args.yaml {
        print!(
            "{}",
            render_yaml(&super::render_data_source_json(
                &datasources,
                (!args.output_columns.is_empty()).then_some(args.output_columns.as_slice()),
            ))?
        );
    } else if args.csv {
        for line in super::render_data_source_csv(
            &datasources,
            (!args.output_columns.is_empty()).then_some(args.output_columns.as_slice()),
        ) {
            println!("{line}");
        }
    } else if args.text {
        for line in super::render_datasource_text(&datasources, &args.output_columns) {
            println!("{line}");
        }
        println!();
        println!("Listed {} data source(s).", datasources.len());
    } else {
        for line in super::render_data_source_table(
            &datasources,
            !args.no_header,
            (!args.output_columns.is_empty()).then_some(args.output_columns.as_slice()),
        ) {
            println!("{line}");
        }
        println!();
        println!("Listed {} data source(s).", datasources.len());
    }
    Ok(())
}

pub(super) fn run_datasource_browse(args: super::DatasourceBrowseArgs) -> Result<()> {
    let _ = super::datasource_browse::browse_datasources(&args)?;
    Ok(())
}
