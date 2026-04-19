//! Datasource runtime import/export/diff/plan handlers.

use serde_json::Value;

use crate::common::{message, string_field, Result};
use crate::dashboard::{
    build_api_client, build_auth_context, build_http_client, build_http_client_for_org_from_api,
};
use crate::grafana_api::DatasourceResourceClient;

pub(super) fn run_datasource_export(mut args: super::DatasourceExportArgs) -> Result<()> {
    args.output_dir =
        Some(super::datasource_runtime_artifacts::resolve_datasource_export_output_dir(&args)?);
    let output_dir = args
        .output_dir
        .as_ref()
        .ok_or_else(|| message("Datasource export output directory was not resolved."))?;
    if args.all_orgs {
        let context = build_auth_context(&args.common)?;
        if context.auth_mode != "basic" {
            return Err(message(
                "Datasource export with --all-orgs requires Basic auth (--basic-user / --basic-password).",
            ));
        }
        let admin_api = build_api_client(&args.common)?;
        let admin_client = admin_api.http_client();
        let admin_datasource = DatasourceResourceClient::new(admin_client);
        let mut total = 0usize;
        let mut org_count = 0usize;
        let mut root_items = Vec::new();
        let mut root_records = Vec::new();
        for org in admin_datasource.list_orgs()? {
            let org_id = org
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| message("Grafana org list entry is missing numeric id."))?;
            let org_id_string = org_id.to_string();
            let org_name = string_field(&org, "name", "");
            let org_client = build_http_client_for_org_from_api(&admin_api, org_id)?;
            let records = super::build_export_records(&org_client)?;
            let scoped_output_dir = super::build_all_orgs_output_dir(output_dir, &org);
            let datasources_path = scoped_output_dir.join(super::DATASOURCE_EXPORT_FILENAME);
            let index_path = scoped_output_dir.join("index.json");
            let metadata_path = scoped_output_dir.join(super::EXPORT_METADATA_FILENAME);
            let provisioning_path = scoped_output_dir
                .join(super::DATASOURCE_PROVISIONING_SUBDIR)
                .join(super::DATASOURCE_PROVISIONING_FILENAME);
            if !args.dry_run {
                super::write_json_file(
                    &datasources_path,
                    &Value::Array(records.clone().into_iter().map(Value::Object).collect()),
                    args.overwrite,
                )?;
                super::write_json_file(
                    &index_path,
                    &super::build_export_index(&records),
                    args.overwrite,
                )?;
                super::write_json_file(
                    &metadata_path,
                    &super::build_datasource_export_metadata(
                        &args.common.url,
                        args.common.profile.as_deref(),
                        Some("org"),
                        Some(&org_id_string),
                        Some(&org_name),
                        &scoped_output_dir,
                        records.len(),
                    ),
                    args.overwrite,
                )?;
                if !args.without_datasource_provisioning {
                    super::write_yaml_file(
                        &provisioning_path,
                        &super::build_datasource_provisioning_document(&records),
                        args.overwrite,
                    )?;
                }
            }
            let summary_verb = if args.dry_run {
                "Would export"
            } else {
                "Exported"
            };
            println!(
                "{summary_verb} {} datasource(s). Datasources: {} Index: {} Manifest: {}{}",
                records.len(),
                datasources_path.display(),
                index_path.display(),
                metadata_path.display(),
                if args.without_datasource_provisioning {
                    String::new()
                } else {
                    format!(" Provisioning: {}", provisioning_path.display())
                }
            );
            for item in super::build_export_index(&records)
                .get("items")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(object) = item.as_object() {
                    let mut entry = object.clone();
                    entry.insert(
                        "exportDir".to_string(),
                        Value::String(scoped_output_dir.display().to_string()),
                    );
                    root_items.push(entry);
                }
            }
            root_records.extend(records.iter().cloned());
            total += records.len();
            org_count += 1;
        }
        if !args.dry_run {
            super::write_json_file(
                &output_dir.join("index.json"),
                &super::build_all_orgs_export_index(&root_items),
                args.overwrite,
            )?;
            super::write_json_file(
                &output_dir.join(super::EXPORT_METADATA_FILENAME),
                &super::build_all_orgs_export_metadata(
                    &args.common.url,
                    args.common.profile.as_deref(),
                    output_dir,
                    org_count,
                    total,
                ),
                args.overwrite,
            )?;
            if !args.without_datasource_provisioning {
                super::write_yaml_file(
                    &output_dir
                        .join(super::DATASOURCE_PROVISIONING_SUBDIR)
                        .join(super::DATASOURCE_PROVISIONING_FILENAME),
                    &super::build_datasource_provisioning_document(&root_records),
                    args.overwrite,
                )?;
            }
        }
        println!(
            "{} datasource(s) across {} exported org(s) under {}",
            total,
            org_count,
            output_dir.display()
        );
        return Ok(());
    }
    let client = super::resolve_target_client(&args.common, args.org_id)?;
    let records = super::build_export_records(&client)?;
    let datasources_path = output_dir.join(super::DATASOURCE_EXPORT_FILENAME);
    let index_path = output_dir.join("index.json");
    let metadata_path = output_dir.join(super::EXPORT_METADATA_FILENAME);
    let provisioning_path = output_dir
        .join(super::DATASOURCE_PROVISIONING_SUBDIR)
        .join(super::DATASOURCE_PROVISIONING_FILENAME);
    if !args.dry_run {
        super::write_json_file(
            &datasources_path,
            &Value::Array(records.clone().into_iter().map(Value::Object).collect()),
            args.overwrite,
        )?;
        super::write_json_file(
            &index_path,
            &super::build_export_index(&records),
            args.overwrite,
        )?;
        super::write_json_file(
            &metadata_path,
            &super::build_datasource_export_metadata(
                &args.common.url,
                args.common.profile.as_deref(),
                Some("org"),
                None,
                None,
                output_dir,
                records.len(),
            ),
            args.overwrite,
        )?;
        if !args.without_datasource_provisioning {
            super::write_yaml_file(
                &provisioning_path,
                &super::build_datasource_provisioning_document(&records),
                args.overwrite,
            )?;
        }
    }
    let summary_verb = if args.dry_run {
        "Would export"
    } else {
        "Exported"
    };
    println!(
        "{summary_verb} {} datasource(s). Datasources: {} Index: {} Manifest: {}{}",
        records.len(),
        datasources_path.display(),
        index_path.display(),
        metadata_path.display(),
        if args.without_datasource_provisioning {
            String::new()
        } else {
            format!(" Provisioning: {}", provisioning_path.display())
        }
    );
    Ok(())
}

pub(super) fn run_datasource_import(mut args: super::DatasourceImportArgs) -> Result<()> {
    if args.local && args.input_dir.is_none() {
        args.input_dir = Some(
            super::datasource_runtime_artifacts::resolve_datasource_artifact_input_dir(
                args.common.profile.as_deref(),
                args.run,
                args.run_id.as_deref(),
            )?,
        );
    }
    super::validate_import_org_auth(&args.common, &args)?;
    if args.table && !args.dry_run {
        return Err(message(
            "--table is only supported with --dry-run for datasource import.",
        ));
    }
    if args.json && !args.dry_run {
        return Err(message(
            "--json is only supported with --dry-run for datasource import.",
        ));
    }
    if args.table && args.json {
        return Err(message(
            "--table and --json are mutually exclusive for datasource import.",
        ));
    }
    if args.no_header && !args.table {
        return Err(message(
            "--no-header is only supported with --dry-run --table for datasource import.",
        ));
    }
    if !args.output_columns.is_empty() && !args.table {
        return Err(message(
            "--output-columns is only supported with --dry-run --table or table-like --output-format for datasource import.",
        ));
    }
    if args.use_export_org {
        if !args.output_columns.is_empty() {
            return Err(message(
                "--output-columns is not supported with --use-export-org for datasource import.",
            ));
        }
        super::import_datasources_by_export_org(&args)?;
        return Ok(());
    }
    let client = super::resolve_target_client(&args.common, args.org_id)?;
    super::import_datasources_with_client(&client, &args)?;
    Ok(())
}

pub(super) fn run_datasource_diff(mut args: super::DatasourceDiffArgs) -> Result<()> {
    if args.local && args.diff_dir.is_none() {
        args.diff_dir = Some(
            super::datasource_runtime_artifacts::resolve_datasource_artifact_input_dir(
                args.common.profile.as_deref(),
                args.run,
                args.run_id.as_deref(),
            )?,
        );
    }
    let client = build_http_client(&args.common)?;
    let datasource_client = DatasourceResourceClient::new(&client);
    let live = datasource_client.list_datasources()?;
    let diff_dir = args
        .diff_dir
        .as_ref()
        .ok_or_else(|| message("Datasource diff requires --diff-dir or --local."))?;
    let (compared_count, differences) =
        super::diff_datasources_with_live(diff_dir, args.input_format, &live, args.output_format)?;
    if differences > 0 {
        return Err(message(format!(
            "Found {} datasource difference(s) across {} exported datasource(s).",
            differences, compared_count
        )));
    }
    println!(
        "No datasource differences across {} exported datasource(s).",
        compared_count
    );
    Ok(())
}

pub(super) fn run_datasource_plan(mut args: super::DatasourcePlanArgs) -> Result<()> {
    if args.local && args.input_dir.is_none() {
        args.input_dir = Some(
            super::datasource_runtime_artifacts::resolve_datasource_artifact_input_dir(
                args.common.profile.as_deref(),
                args.run,
                args.run_id.as_deref(),
            )?,
        );
    }
    let plan_input = if args.use_export_org {
        build_routed_datasource_plan_input(&args)?
    } else {
        let client = super::resolve_target_client(&args.common, args.org_id)?;
        let input_dir_arg = args
            .input_dir
            .as_ref()
            .ok_or_else(|| message("Datasource plan requires --input-dir or --local."))?;
        let input_dir = if args.input_format == super::DatasourceImportInputFormat::Inventory {
            super::resolve_datasource_export_root_dir(input_dir_arg)?
        } else {
            input_dir_arg.clone()
        };
        let (_metadata, records) = super::load_import_records(&input_dir, args.input_format)?;
        let live = DatasourceResourceClient::new(&client).list_datasources()?;
        super::DatasourcePlanInput {
            scope: if args.org_id.is_some() {
                "explicit-org".to_string()
            } else {
                "current-org".to_string()
            },
            input_format: format!("{:?}", args.input_format).to_ascii_lowercase(),
            prune: args.prune,
            orgs: vec![super::DatasourcePlanOrgInput {
                source_org_id: String::new(),
                source_org_name: String::new(),
                target_org_id: args.org_id.map(|value| value.to_string()),
                target_org_name: String::new(),
                org_action: "current".to_string(),
                input_dir,
                records,
                live,
            }],
        }
    };
    let report = super::build_datasource_plan(plan_input);
    super::print_datasource_plan_report(
        &report,
        args.output_format,
        args.show_same,
        !args.no_header,
        &args.output_columns,
    )
}

pub(super) fn build_routed_datasource_plan_input(
    args: &super::DatasourcePlanArgs,
) -> Result<super::DatasourcePlanInput> {
    let context = build_auth_context(&args.common)?;
    if context.auth_mode != "basic" {
        return Err(message(
            "Datasource plan with --use-export-org requires Basic auth (--basic-user / --basic-password).",
        ));
    }
    let admin_api = build_api_client(&args.common)?;
    let admin_client = admin_api.http_client();
    let import_args = datasource_plan_to_import_args(args);
    let scopes = super::discover_export_org_import_scopes(&import_args)?;
    let mut orgs = Vec::new();
    for scope in scopes {
        let target_plan =
            super::resolve_export_org_target_plan(admin_client, &import_args, &scope)?;
        let (_metadata, records) =
            super::load_import_records(&target_plan.input_dir, args.input_format)?;
        let live = if let Some(target_org_id) = target_plan.target_org_id {
            let org_client = build_http_client_for_org_from_api(&admin_api, target_org_id)?;
            DatasourceResourceClient::new(&org_client).list_datasources()?
        } else {
            Vec::new()
        };
        orgs.push(super::DatasourcePlanOrgInput {
            source_org_id: target_plan.source_org_id.to_string(),
            source_org_name: target_plan.source_org_name,
            target_org_id: target_plan.target_org_id.map(|value| value.to_string()),
            target_org_name: String::new(),
            org_action: target_plan.org_action.to_string(),
            input_dir: target_plan.input_dir,
            records,
            live,
        });
    }
    Ok(super::DatasourcePlanInput {
        scope: "export-orgs".to_string(),
        input_format: format!("{:?}", args.input_format).to_ascii_lowercase(),
        prune: args.prune,
        orgs,
    })
}

pub(super) fn datasource_plan_to_import_args(
    args: &super::DatasourcePlanArgs,
) -> super::DatasourceImportArgs {
    super::DatasourceImportArgs {
        common: args.common.clone(),
        input_dir: args.input_dir.clone(),
        local: args.local,
        run: args.run,
        run_id: args.run_id.clone(),
        input_format: args.input_format,
        org_id: args.org_id,
        use_export_org: args.use_export_org,
        only_org_id: args.only_org_id.clone(),
        create_missing_orgs: args.create_missing_orgs,
        require_matching_export_org: false,
        replace_existing: false,
        update_existing_only: false,
        secret_values: None,
        secret_values_file: None,
        dry_run: true,
        table: false,
        json: false,
        output_format: None,
        no_header: false,
        output_columns: Vec::new(),
        list_columns: false,
        progress: false,
        verbose: false,
    }
}
