//! Datasource command runtime facade.
//!
//! Purpose:
//! - Normalize datasource commands before execution.
//! - Handle early exits and command-shape validation.
//! - Materialize auth once before dispatching to command handlers.

use crate::common::Result;

use super::DatasourceGroupCommand;

// Datasource runtime boundary:
// normalize shared flags, validate, materialize auth, then dispatch by command kind.
pub fn run_datasource_cli(command: DatasourceGroupCommand) -> Result<()> {
    // Runtime boundary for datasource commands:
    // normalize legacy flags once, apply command-only exits, then materialize auth and execute.
    let command = super::normalize_datasource_group_command(command);
    if super::datasource_runtime_guardrails::handle_datasource_command_early_exits(&command)? {
        return Ok(());
    }
    super::datasource_runtime_guardrails::validate_datasource_command_inputs(&command)?;
    let command =
        super::datasource_runtime_guardrails::materialize_datasource_command_auth(command)?;
    execute_datasource_command(command)
}

fn execute_datasource_command(command: DatasourceGroupCommand) -> Result<()> {
    match command {
        DatasourceGroupCommand::Types(args) => {
            super::datasource_runtime_list::run_datasource_types(args)
        }
        DatasourceGroupCommand::List(args) => {
            super::datasource_runtime_list::run_datasource_list(args)
        }
        DatasourceGroupCommand::Browse(args) => {
            super::datasource_runtime_list::run_datasource_browse(args)
        }
        DatasourceGroupCommand::Add(args) => {
            super::datasource_runtime_mutation::run_datasource_add(args)
        }
        DatasourceGroupCommand::Modify(args) => {
            super::datasource_runtime_mutation::run_datasource_modify(args)
        }
        DatasourceGroupCommand::Delete(args) => {
            super::datasource_runtime_mutation::run_datasource_delete(args)
        }
        DatasourceGroupCommand::Export(args) => {
            super::datasource_runtime_sync::run_datasource_export(args)
        }
        DatasourceGroupCommand::Import(args) => {
            super::datasource_runtime_sync::run_datasource_import(args)
        }
        DatasourceGroupCommand::Diff(args) => {
            super::datasource_runtime_sync::run_datasource_diff(args)
        }
        DatasourceGroupCommand::Plan(args) => {
            super::datasource_runtime_sync::run_datasource_plan(args)
        }
    }
}
