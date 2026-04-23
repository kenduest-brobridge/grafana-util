use serde_json::Value;

use crate::common::Result;
use crate::sync::live::SyncApplyOperation;

use super::sync_live_apply_execution::apply_live_operation_with_client;
use super::sync_live_apply_phase::execute_live_apply_phase;
use super::SyncLiveClient;

#[cfg(test)]
#[path = "sync_live_apply_request.rs"]
mod sync_live_apply_request;
#[cfg(test)]
pub(crate) use sync_live_apply_request::execute_live_apply_with_request;

impl<'a> SyncLiveClient<'a> {
    pub(crate) fn execute_live_apply(
        &self,
        operations: &[SyncApplyOperation],
        allow_folder_delete: bool,
        allow_policy_reset: bool,
    ) -> Result<Value> {
        execute_live_apply_phase(operations, allow_policy_reset, |operation| {
            apply_live_operation_with_client(self, operation, allow_folder_delete)
        })
    }
}

pub(crate) fn execute_live_apply_with_client(
    client: &SyncLiveClient<'_>,
    operations: &[SyncApplyOperation],
    allow_folder_delete: bool,
    allow_policy_reset: bool,
) -> Result<Value> {
    client.execute_live_apply(operations, allow_folder_delete, allow_policy_reset)
}
