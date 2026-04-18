use clap::ValueEnum;

use super::DatasourceGroupCommand;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ListOutputFormat {
    Text,
    Table,
    Csv,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DryRunOutputFormat {
    Text,
    Table,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DatasourcePlanOutputFormat {
    Text,
    Table,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DatasourceImportInputFormat {
    Inventory,
    Provisioning,
}

pub(crate) fn parse_datasource_list_output_column(
    value: &str,
) -> std::result::Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("--output-columns values must not be empty.".to_string());
    }
    Ok(match trimmed {
        "all" => "all".to_string(),
        "isDefault" => "is_default".to_string(),
        "orgId" => "org_id".to_string(),
        other => other.to_string(),
    })
}

pub(crate) fn parse_datasource_import_output_column(
    value: &str,
) -> std::result::Result<String, String> {
    match value {
        "all" => Ok("all".to_string()),
        "uid" => Ok("uid".to_string()),
        "name" => Ok("name".to_string()),
        "type" => Ok("type".to_string()),
        "match_basis" | "matchBasis" => Ok("match_basis".to_string()),
        "destination" => Ok("destination".to_string()),
        "action" => Ok("action".to_string()),
        "org_id" | "orgId" => Ok("org_id".to_string()),
        "file" => Ok("file".to_string()),
        "target_uid" | "targetUid" => Ok("target_uid".to_string()),
        "target_version" | "targetVersion" => Ok("target_version".to_string()),
        "target_read_only" | "targetReadOnly" => Ok("target_read_only".to_string()),
        "blocked_reason" | "blockedReason" => Ok("blocked_reason".to_string()),
        _ => Err(format!(
            "Unsupported --output-columns value '{value}'. Supported values: all, uid, name, type, match_basis, destination, action, org_id, file, target_uid, target_version, target_read_only, blocked_reason."
        )),
    }
}

pub(crate) fn parse_datasource_plan_output_column(
    value: &str,
) -> std::result::Result<String, String> {
    match value {
        "all" => Ok("all".to_string()),
        "action_id" | "actionId" => Ok("action_id".to_string()),
        "action" => Ok("action".to_string()),
        "status" => Ok("status".to_string()),
        "uid" => Ok("uid".to_string()),
        "name" => Ok("name".to_string()),
        "type" => Ok("type".to_string()),
        "match_basis" | "matchBasis" => Ok("match_basis".to_string()),
        "source_org_id" | "sourceOrgId" => Ok("source_org_id".to_string()),
        "target_org_id" | "targetOrgId" => Ok("target_org_id".to_string()),
        "target_uid" | "targetUid" => Ok("target_uid".to_string()),
        "target_version" | "targetVersion" => Ok("target_version".to_string()),
        "target_read_only" | "targetReadOnly" => Ok("target_read_only".to_string()),
        "changed_fields" | "changedFields" => Ok("changed_fields".to_string()),
        "blocked_reason" | "blockedReason" => Ok("blocked_reason".to_string()),
        "source_file" | "sourceFile" => Ok("source_file".to_string()),
        _ => Err(format!(
            "Unsupported --output-columns value '{value}'. Supported values: all, action_id, action, status, uid, name, type, match_basis, source_org_id, target_org_id, target_uid, target_version, target_read_only, changed_fields, blocked_reason, source_file."
        )),
    }
}

pub(crate) fn parse_bool_choice(value: &str) -> std::result::Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("value must be true or false".to_string()),
    }
}

#[cfg(test)]
pub(crate) fn normalize_output_formats(args: &mut super::DatasourceCliArgs) {
    apply_output_format(&mut args.command);
}

pub(crate) fn normalize_datasource_group_command(
    mut command: DatasourceGroupCommand,
) -> DatasourceGroupCommand {
    apply_output_format(&mut command);
    command
}

fn apply_output_format(command: &mut DatasourceGroupCommand) {
    match command {
        DatasourceGroupCommand::List(inner) => match inner.output_format {
            Some(ListOutputFormat::Text) => inner.text = true,
            Some(ListOutputFormat::Table) => inner.table = true,
            Some(ListOutputFormat::Csv) => inner.csv = true,
            Some(ListOutputFormat::Json) => inner.json = true,
            Some(ListOutputFormat::Yaml) => inner.yaml = true,
            None => {}
        },
        DatasourceGroupCommand::Browse(_) => {}
        DatasourceGroupCommand::Import(inner) => {
            apply_dry_run_output_format(inner.output_format, &mut inner.table, &mut inner.json)
        }
        DatasourceGroupCommand::Add(inner) => {
            apply_dry_run_output_format(inner.output_format, &mut inner.table, &mut inner.json)
        }
        DatasourceGroupCommand::Modify(inner) => {
            apply_dry_run_output_format(inner.output_format, &mut inner.table, &mut inner.json)
        }
        DatasourceGroupCommand::Delete(inner) => {
            apply_dry_run_output_format(inner.output_format, &mut inner.table, &mut inner.json)
        }
        _ => {}
    }
}

fn apply_dry_run_output_format(
    output_format: Option<DryRunOutputFormat>,
    table: &mut bool,
    json: &mut bool,
) {
    match output_format {
        Some(DryRunOutputFormat::Table) => *table = true,
        Some(DryRunOutputFormat::Json) => *json = true,
        Some(DryRunOutputFormat::Text) | None => {}
    }
}
