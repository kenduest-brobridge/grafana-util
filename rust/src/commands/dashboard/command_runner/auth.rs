use crate::common::Result;

use super::super::{
    materialize_dashboard_common_auth, DashboardCliArgs, DashboardCommand,
    DashboardHistorySubcommand,
};

pub(crate) fn materialize_dashboard_command_auth(args: &mut DashboardCliArgs) -> Result<()> {
    match &mut args.command {
        DashboardCommand::Browse(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::List(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Export(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Get(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::CloneLive(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::EditLive(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Import(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Plan(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::InspectLive(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Diff(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Screenshot(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Delete(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::Publish(inner) => {
            inner.common = materialize_dashboard_common_auth(inner.common.clone())?
        }
        DashboardCommand::History(history_args) => match &mut history_args.command {
            DashboardHistorySubcommand::List(inner) => {
                inner.common = materialize_dashboard_common_auth(inner.common.clone())?
            }
            DashboardHistorySubcommand::Restore(inner) => {
                inner.common = materialize_dashboard_common_auth(inner.common.clone())?
            }
            DashboardHistorySubcommand::Export(inner) => {
                inner.common = materialize_dashboard_common_auth(inner.common.clone())?
            }
            DashboardHistorySubcommand::Diff(_) => {}
        },
        DashboardCommand::Review(_)
        | DashboardCommand::PatchFile(_)
        | DashboardCommand::Serve(_)
        | DashboardCommand::Summary(_)
        | DashboardCommand::GovernanceGate(_)
        | DashboardCommand::Topology(_)
        | DashboardCommand::Impact(_)
        | DashboardCommand::ValidateExport(_)
        | DashboardCommand::InspectExport(_)
        | DashboardCommand::InspectVars(_) => {}
    }
    Ok(())
}
