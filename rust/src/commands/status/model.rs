//! Shared status producer model.
//!
//! Maintainer note:
//! - This module owns the small reusable data shape that sits between raw
//!   producer inputs and the existing project-status contract.
//! - Keep it boring: data shape first, no trait layer yet, no orchestration.

use serde::Serialize;

use crate::project_status::{
    ProjectDomainStatus, ProjectDomainStatusReading, ProjectStatusFinding, ProjectStatusFreshness,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusRecordCount {
    pub kind: String,
    pub count: usize,
    pub source: String,
}

pub type StatusWarning = StatusRecordCount;
pub type StatusBlockedReason = StatusRecordCount;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusReading {
    pub id: String,
    pub scope: String,
    pub mode: String,
    pub status: String,
    pub reason_code: String,
    pub primary_count: usize,
    pub source_kinds: Vec<String>,
    pub signal_keys: Vec<String>,
    pub blockers: Vec<StatusBlockedReason>,
    pub warnings: Vec<StatusWarning>,
    pub next_actions: Vec<String>,
    pub freshness: ProjectStatusFreshness,
}

impl StatusRecordCount {
    pub fn new(kind: &str, count: usize, source: &str) -> Self {
        Self {
            kind: kind.to_string(),
            count,
            source: source.to_string(),
        }
    }
}

impl From<ProjectStatusFinding> for StatusRecordCount {
    fn from(value: ProjectStatusFinding) -> Self {
        Self {
            kind: value.kind,
            count: value.count,
            source: value.source,
        }
    }
}

impl From<StatusRecordCount> for ProjectStatusFinding {
    fn from(value: StatusRecordCount) -> Self {
        ProjectStatusFinding {
            kind: value.kind,
            count: value.count,
            source: value.source,
        }
    }
}

impl StatusReading {
    pub fn into_project_domain_status_reading(self) -> ProjectDomainStatusReading {
        ProjectDomainStatusReading {
            id: self.id,
            scope: self.scope,
            mode: self.mode,
            status: self.status,
            reason_code: self.reason_code,
            primary_count: self.primary_count,
            source_kinds: self.source_kinds,
            signal_keys: self.signal_keys,
            blockers: self.blockers.into_iter().map(Into::into).collect(),
            warnings: self.warnings.into_iter().map(Into::into).collect(),
            next_actions: self.next_actions,
            freshness: self.freshness,
        }
    }

    pub fn into_project_domain_status(self) -> ProjectDomainStatus {
        self.into_project_domain_status_reading()
            .into_domain_status()
    }
}

#[cfg(test)]
mod tests {
    use super::{StatusReading, StatusRecordCount};
    use crate::project_status::{PROJECT_STATUS_BLOCKED, PROJECT_STATUS_READY};

    #[test]
    fn status_reading_derives_project_domain_status_counts() {
        let status = StatusReading {
            id: "datasource".to_string(),
            scope: "staged".to_string(),
            mode: "artifact-summary".to_string(),
            status: PROJECT_STATUS_BLOCKED.to_string(),
            reason_code: "blocked-by-blockers".to_string(),
            primary_count: 3,
            source_kinds: vec!["datasource-export".to_string()],
            signal_keys: vec!["summary.datasourceCount".to_string()],
            blockers: vec![StatusRecordCount::new(
                "missing-default",
                2,
                "summary.defaultCount",
            )],
            warnings: vec![StatusRecordCount::new("diff-drift", 4, "summary.diffCount")],
            next_actions: vec!["review datasource diff drift before import or sync".to_string()],
            freshness: Default::default(),
        }
        .into_project_domain_status();

        assert_eq!(status.status, PROJECT_STATUS_BLOCKED);
        assert_eq!(status.blocker_count, 2);
        assert_eq!(status.warning_count, 4);
        assert_eq!(status.reason_code, "blocked-by-blockers");
    }

    #[test]
    fn status_record_count_round_trips_through_project_status_finding() {
        let record = StatusRecordCount::new("ready", 1, "summary.ruleCount");
        let finding = crate::project_status::ProjectStatusFinding::from(record.clone());

        assert_eq!(finding.kind, "ready");
        assert_eq!(finding.count, 1);
        assert_eq!(finding.source, "summary.ruleCount");

        let record_again = StatusRecordCount::from(finding);
        assert_eq!(record_again, record);
        let _ = PROJECT_STATUS_READY;
    }
}
