//! Apply-phase sequencing for already ordered live apply intent operations.
//!
//! Dependency-aware ordering is owned by sync plan construction. This phase
//! must preserve the reviewed apply intent order and only enforce live-apply
//! guards that belong at execution time.

use serde_json::Value;

use crate::common::Result;
use crate::review_contract::REVIEW_ACTION_WOULD_DELETE;
use crate::sync::live::SyncApplyOperation;

use super::sync_live_apply_error::refuse_live_policy_reset;
use super::sync_live_apply_result::{
    append_live_apply_result, finish_live_apply_response, normalize_live_apply_result,
};

pub(crate) fn execute_live_apply_phase<F>(
    operations: &[SyncApplyOperation],
    allow_policy_reset: bool,
    mut apply_operation: F,
) -> Result<Value>
where
    F: FnMut(&SyncApplyOperation) -> Result<Value>,
{
    let mut results = Vec::new();
    for operation in operations {
        if operation.kind == "alert-policy"
            && operation.action == REVIEW_ACTION_WOULD_DELETE
            && !allow_policy_reset
        {
            return Err(refuse_live_policy_reset());
        }
        let response = apply_operation(operation)?;
        let normalized = normalize_live_apply_result(operation, response);
        append_live_apply_result(&mut results, normalized);
    }
    Ok(finish_live_apply_response(results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review_contract::{REVIEW_ACTION_WOULD_DELETE, REVIEW_ACTION_WOULD_UPDATE};
    use serde_json::json;

    fn operation(kind: &str, action: &str, identity: &str) -> SyncApplyOperation {
        SyncApplyOperation {
            kind: kind.to_string(),
            identity: identity.to_string(),
            action: action.to_string(),
            desired: serde_json::Map::new(),
            ownership: String::new(),
            provenance: Vec::new(),
        }
    }

    #[test]
    fn phase_preserves_apply_intent_order_without_reordering_by_dependency_rank() {
        let operations = vec![
            operation("dashboard", REVIEW_ACTION_WOULD_UPDATE, "dash-a"),
            operation("folder", REVIEW_ACTION_WOULD_UPDATE, "folder-a"),
            operation("alert", REVIEW_ACTION_WOULD_DELETE, "alert-a"),
        ];
        let mut visited = Vec::new();
        let result = execute_live_apply_phase(&operations, false, |op| {
            visited.push(format!("{}:{}:{}", op.action, op.kind, op.identity));
            Ok(json!({
                "kind": op.kind,
                "identity": op.identity,
            }))
        })
        .unwrap();

        assert_eq!(result["mode"], json!("live-apply"));
        assert_eq!(result["appliedCount"], json!(3));
        assert_eq!(
            visited,
            vec![
                "would-update:dashboard:dash-a",
                "would-update:folder:folder-a",
                "would-delete:alert:alert-a",
            ]
        );
        assert_eq!(result["results"][0]["kind"], json!("dashboard"));
        assert_eq!(result["results"][0]["identity"], json!("dash-a"));
        assert_eq!(result["results"][1]["kind"], json!("folder"));
        assert_eq!(result["results"][1]["identity"], json!("folder-a"));
        assert_eq!(result["results"][2]["kind"], json!("alert"));
        assert_eq!(result["results"][2]["identity"], json!("alert-a"));
    }

    #[test]
    fn phase_blocks_policy_reset_when_not_allowed() {
        let operations = vec![operation(
            "alert-policy",
            REVIEW_ACTION_WOULD_DELETE,
            "policies",
        )];
        let result =
            execute_live_apply_phase(&operations, false, |_| Ok(json!({"should_not_run": true})));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Refusing live notification policy reset without --allow-policy-reset."
        );
    }
}
