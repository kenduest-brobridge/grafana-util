//! Live datasource domain-status producer.
//!
//! Maintainer note:
//! - This module now acts as the thin orchestration wrapper around the
//!   datasource live-status analysis helpers.
//! - Keep it conservative and source-attributable: prefer the live list
//!   response, fall back to a single live read response when needed, and only
//!   derive counts that are directly visible in the payloads.

use serde_json::Value;

use crate::common::Result;
use crate::grafana_api::datasource_live_project_status as datasource_live_project_status_support;

#[path = "live_analysis.rs"]
mod analysis;

#[allow(unused_imports)]
pub(crate) use analysis::{
    build_datasource_live_project_status, build_datasource_live_project_status_from_inputs,
    datasource_live_project_status_org_count, datasource_live_project_status_source_kinds,
    DatasourceLiveProjectStatusInputs, LiveDatasourceProjectStatusInputs,
};

pub(crate) fn collect_live_datasource_project_status_inputs_with_request<F>(
    request_json: &mut F,
) -> Result<LiveDatasourceProjectStatusInputs>
where
    F: FnMut(reqwest::Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    datasource_live_project_status_support::collect_live_datasource_project_status_inputs_with_request(
        request_json,
    )
}

#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
