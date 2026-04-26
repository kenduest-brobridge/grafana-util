use serde_json::Value;

use crate::review_contract::{review_apply_result_entry, ReviewApplyResult};
use crate::sync::live::SyncApplyOperation;

pub(crate) fn normalize_live_apply_result(
    operation: &SyncApplyOperation,
    response: Value,
) -> Value {
    review_apply_result_entry(
        operation.kind.as_str(),
        operation.identity.as_str(),
        operation.action.as_str(),
        response,
    )
}

pub(crate) fn append_live_apply_result(results: &mut Vec<Value>, result: Value) {
    results.push(result);
}

pub(crate) fn finish_live_apply_response(results: Vec<Value>) -> Value {
    ReviewApplyResult::from_results("live-apply", results).into_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review_contract::REVIEW_ACTION_WOULD_UPDATE;
    use crate::sync::live::SyncApplyOperation;
    use serde_json::json;

    #[test]
    fn normalize_live_apply_result_preserves_operation_identity_and_response() {
        let operation = SyncApplyOperation {
            kind: "dashboard".to_string(),
            identity: "dash-uid".to_string(),
            action: REVIEW_ACTION_WOULD_UPDATE.to_string(),
            desired: serde_json::Map::new(),
            ownership: String::new(),
            provenance: Vec::new(),
        };
        let result = normalize_live_apply_result(&operation, json!({"status":"ok"}));

        assert_eq!(result["kind"], json!("dashboard"));
        assert_eq!(result["identity"], json!("dash-uid"));
        assert_eq!(result["action"], json!("would-update"));
        assert_eq!(result["response"]["status"], json!("ok"));
    }
}
