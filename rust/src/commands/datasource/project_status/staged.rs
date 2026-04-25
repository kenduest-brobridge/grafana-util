//! Datasource staged domain-status wrapper.
//!
//! Maintainer note:
//! - Keep this module as a thin wrapper around the staged-reading producer.
//! - The concrete producer/model logic lives in `staged_reading`.

use serde_json::Value;

use crate::project_status::ProjectDomainStatus;
use crate::project_status_model::StatusProducer;

#[path = "staged_reading.rs"]
mod reading;

pub(crate) fn build_datasource_domain_status(
    summary_document: Option<&Value>,
) -> Option<ProjectDomainStatus> {
    reading::DatasourceDomainStatusInputs { summary_document }.project_domain_status()
}
