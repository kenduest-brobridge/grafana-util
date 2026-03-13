use serde_json::{Map, Value};

use super::dashboard_inspect::QueryAnalysis;

pub(crate) fn analyze_query(
    _panel: &Map<String, Value>,
    _target: &Map<String, Value>,
    _query_field: &str,
    _query_text: &str,
) -> QueryAnalysis {
    QueryAnalysis::default()
}
