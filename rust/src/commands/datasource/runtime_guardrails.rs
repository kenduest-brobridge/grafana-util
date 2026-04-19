//! Datasource runtime validation, early exits, and auth materialization.

use crate::common::{message, print_supported_columns, Result};
use crate::dashboard::materialize_dashboard_common_auth;

use super::DatasourceGroupCommand;

const DATASOURCE_IMPORT_LIST_COLUMNS: &[&str] = &[
    "uid",
    "name",
    "type",
    "match_basis",
    "destination",
    "action",
    "org_id",
    "file",
    "target_uid",
    "target_version",
    "target_read_only",
    "blocked_reason",
];

pub(super) fn handle_datasource_command_early_exits(
    command: &DatasourceGroupCommand,
) -> Result<bool> {
    match command {
        DatasourceGroupCommand::List(args) if args.list_columns => {
            print_supported_columns(super::datasource_list_column_ids());
            Ok(true)
        }
        DatasourceGroupCommand::Import(args) if args.list_columns => {
            print_supported_columns(DATASOURCE_IMPORT_LIST_COLUMNS);
            Ok(true)
        }
        DatasourceGroupCommand::Plan(args) if args.list_columns => {
            print_supported_columns(super::datasource_plan_column_ids());
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn validate_datasource_command_inputs(command: &DatasourceGroupCommand) -> Result<()> {
    match command {
        DatasourceGroupCommand::Import(args) if !args.output_columns.is_empty() && !args.table => {
            return Err(message(
                "--output-columns is only supported with --dry-run --table or table-like --output-format for datasource import.",
            ));
        }
        DatasourceGroupCommand::Plan(args) => {
            if !args.output_columns.is_empty()
                && args.output_format != super::DatasourcePlanOutputFormat::Table
            {
                return Err(message(
                    "--output-columns is only supported with --output-format table for datasource plan.",
                ));
            }
            if args.no_header && args.output_format != super::DatasourcePlanOutputFormat::Table {
                return Err(message(
                    "--no-header is only supported with --output-format table for datasource plan.",
                ));
            }
            if args.use_export_org
                && args.input_format != super::DatasourceImportInputFormat::Inventory
            {
                return Err(message(
                    "Datasource plan with --use-export-org requires --input-format inventory.",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn materialize_datasource_command_auth(
    mut command: DatasourceGroupCommand,
) -> Result<DatasourceGroupCommand> {
    match &mut command {
        DatasourceGroupCommand::List(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Add(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Modify(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Delete(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Export(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Import(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Diff(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Plan(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Browse(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?;
        }
        DatasourceGroupCommand::Types(_) => {}
    }
    Ok(command)
}
