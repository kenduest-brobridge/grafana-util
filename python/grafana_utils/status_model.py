"""Shared status producer model."""

from dataclasses import dataclass, field
from typing import List
from .project_status import (
    ProjectDomainStatus,
    ProjectStatusFinding,
    ProjectStatusFreshness,
)


@dataclass
class StatusRecordCount:
    kind: str
    count: int
    source: str

    def to_project_status_finding(self) -> ProjectStatusFinding:
        return ProjectStatusFinding(
            kind=self.kind,
            count=self.count,
            source=self.source,
        )


StatusWarning = StatusRecordCount
StatusBlockedReason = StatusRecordCount


@dataclass
class StatusReading:
    id: str
    scope: str
    mode: str
    status: str
    reason_code: str
    primary_count: int
    source_kinds: List[str]
    signal_keys: List[str]
    blockers: List[StatusBlockedReason]
    warnings: List[StatusWarning]
    next_actions: List[str]
    freshness: ProjectStatusFreshness = field(default_factory=ProjectStatusFreshness)

    def into_project_domain_status(self) -> ProjectDomainStatus:
        blocker_count = sum(b.count for b in self.blockers)
        warning_count = sum(w.count for w in self.warnings)
        return ProjectDomainStatus(
            id=self.id,
            scope=self.scope,
            mode=self.mode,
            status=self.status,
            reason_code=self.reason_code,
            primary_count=self.primary_count,
            blocker_count=blocker_count,
            warning_count=warning_count,
            source_kinds=self.source_kinds,
            signal_keys=self.signal_keys,
            blockers=[b.to_project_status_finding() for b in self.blockers],
            warnings=[w.to_project_status_finding() for w in self.warnings],
            next_actions=self.next_actions,
            freshness=self.freshness,
        )
