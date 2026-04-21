//! Datasource runtime live mutation handlers.

use serde_json::Value;

use crate::common::{message, render_json_value, string_field, Result};
use crate::dashboard::build_http_client;
use crate::grafana_api::DatasourceResourceClient;

pub(super) fn run_datasource_add(args: super::DatasourceAddArgs) -> Result<()> {
    super::validate_live_mutation_dry_run_args(
        args.table,
        args.json,
        args.dry_run,
        args.no_header,
        "add",
    )?;
    let payload = super::build_add_payload(&args)?;
    let client = build_http_client(&args.common)?;
    let datasource_client = DatasourceResourceClient::new(&client);
    let live = datasource_client.list_datasources()?;
    let matching = super::resolve_live_mutation_match(args.uid.as_deref(), Some(&args.name), &live);
    let row = vec![
        "add".to_string(),
        args.uid.clone().unwrap_or_default(),
        args.name.clone(),
        args.datasource_type.clone(),
        matching.destination.to_string(),
        matching.action.to_string(),
        matching
            .target_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    ];
    if args.dry_run {
        if args.json {
            print!(
                "{}",
                render_json_value(&super::render_live_mutation_json(&[row]))?
            );
        } else if args.table {
            for line in super::render_live_mutation_table(&[row], !args.no_header) {
                println!("{line}");
            }
            println!("Dry-run checked 1 datasource add request");
        } else {
            println!(
                "Dry-run datasource add uid={} name={} match={} action={}",
                args.uid.clone().unwrap_or_default(),
                args.name,
                matching.destination,
                matching.action
            );
            println!("Dry-run checked 1 datasource add request");
        }
        return Ok(());
    }
    if matching.action != "would-create" {
        return Err(message(format!(
            "Datasource add blocked for name={} uid={}: destination={} action={}.",
            args.name,
            args.uid.clone().unwrap_or_default(),
            matching.destination,
            matching.action
        )));
    }
    datasource_client.create_datasource(
        payload
            .as_object()
            .ok_or_else(|| message("Datasource add payload must be an object."))?,
    )?;
    println!(
        "Created datasource uid={} name={}",
        args.uid.unwrap_or_default(),
        args.name
    );
    Ok(())
}

pub(super) fn run_datasource_modify(args: super::DatasourceModifyArgs) -> Result<()> {
    super::validate_live_mutation_dry_run_args(
        args.table,
        args.json,
        args.dry_run,
        args.no_header,
        "modify",
    )?;
    let updates = super::build_modify_updates(&args)?;
    let client = build_http_client(&args.common)?;
    let datasource_client = DatasourceResourceClient::new(&client);
    let existing = super::fetch_datasource_by_uid_if_exists(&client, &args.uid)?;
    let (action, destination, payload, name, datasource_type, target_id) =
        if let Some(existing) = existing {
            let payload = super::build_modify_payload(&existing, &updates)?;
            (
                "would-update",
                "exists-uid",
                Some(payload),
                string_field(&existing, "name", ""),
                string_field(&existing, "type", ""),
                existing.get("id").and_then(Value::as_i64),
            )
        } else {
            (
                "would-fail-missing",
                "missing",
                None,
                String::new(),
                String::new(),
                None,
            )
        };
    let row = vec![
        "modify".to_string(),
        args.uid.clone(),
        name.clone(),
        datasource_type.clone(),
        destination.to_string(),
        action.to_string(),
        target_id.map(|id| id.to_string()).unwrap_or_default(),
    ];
    if args.dry_run {
        if args.json {
            print!(
                "{}",
                render_json_value(&super::render_live_mutation_json(&[row]))?
            );
        } else if args.table {
            for line in super::render_live_mutation_table(&[row], !args.no_header) {
                println!("{line}");
            }
            println!("Dry-run checked 1 datasource modify request");
        } else {
            println!(
                "Dry-run datasource modify uid={} name={} match={} action={}",
                args.uid, name, destination, action
            );
            println!("Dry-run checked 1 datasource modify request");
        }
        return Ok(());
    }
    if action != "would-update" {
        return Err(message(format!(
            "Datasource modify blocked for uid={}: destination={} action={}.",
            args.uid, destination, action
        )));
    }
    let payload = payload.ok_or_else(|| message("Datasource modify did not build a payload."))?;
    datasource_client.update_datasource_by_uid(
        &args.uid,
        target_id.map(|id| id.to_string()).as_deref(),
        payload
            .as_object()
            .ok_or_else(|| message("Datasource modify payload must be an object."))?,
    )?;
    println!(
        "Modified datasource uid={} name={} id={}",
        args.uid,
        name,
        target_id.map(|id| id.to_string()).unwrap_or_default()
    );
    Ok(())
}

pub(super) fn run_datasource_delete(args: super::DatasourceDeleteArgs) -> Result<()> {
    super::validate_live_mutation_dry_run_args(
        args.table,
        args.json,
        args.dry_run,
        args.no_header,
        "delete",
    )?;
    let client = build_http_client(&args.common)?;
    let datasource_client = DatasourceResourceClient::new(&client);
    let live = datasource_client.list_datasources()?;
    let matching = super::resolve_delete_match(args.uid.as_deref(), args.name.as_deref(), &live);
    let delete_type = super::resolve_delete_preview_type(matching.target_id, &live);
    let row = vec![
        "delete".to_string(),
        args.uid
            .clone()
            .or_else(|| {
                if matching.target_uid.is_empty() {
                    None
                } else {
                    Some(matching.target_uid.clone())
                }
            })
            .unwrap_or_default(),
        args.name
            .clone()
            .unwrap_or_else(|| matching.target_name.clone()),
        delete_type.clone(),
        matching.destination.to_string(),
        matching.action.to_string(),
        matching
            .target_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
    ];
    if args.dry_run {
        if args.json {
            print!(
                "{}",
                render_json_value(&super::render_live_mutation_json(&[row]))?
            );
        } else if args.table {
            for line in super::render_live_mutation_table(&[row], !args.no_header) {
                println!("{line}");
            }
            println!("Dry-run checked 1 datasource delete request");
        } else {
            println!(
                "Dry-run datasource delete uid={} name={} type={} match={} action={}",
                args.uid.clone().unwrap_or_default(),
                args.name.clone().unwrap_or_default(),
                delete_type,
                matching.destination,
                matching.action
            );
            println!("Dry-run checked 1 datasource delete request");
        }
        return Ok(());
    }
    if !args.yes {
        return Err(message(
            "Datasource delete requires --yes unless --dry-run is set.",
        ));
    }
    if matching.action != "would-delete" {
        return Err(message(format!(
            "Datasource delete blocked for uid={} name={} type={}: destination={} action={}.",
            args.uid.clone().unwrap_or_default(),
            args.name.clone().unwrap_or_default(),
            super::resolve_delete_preview_type(matching.target_id, &live),
            matching.destination,
            matching.action
        )));
    }
    let target_id = matching
        .target_id
        .ok_or_else(|| message("Datasource delete requires a live datasource id."))?;
    let target_uid = if matching.target_uid.is_empty() {
        args.uid.as_deref().unwrap_or_default()
    } else {
        matching.target_uid.as_str()
    };
    if target_uid.is_empty() {
        return Err(message("Datasource delete requires a live datasource uid."));
    }
    datasource_client.delete_datasource_by_uid(target_uid, Some(&target_id.to_string()))?;
    println!(
        "Deleted datasource uid={} name={} type={} id={}",
        target_uid,
        if matching.target_name.is_empty() {
            args.name.unwrap_or_default()
        } else {
            matching.target_name
        },
        super::resolve_delete_preview_type(Some(target_id), &live),
        target_id
    );
    Ok(())
}
