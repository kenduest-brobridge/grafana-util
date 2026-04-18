"""Shared status contract types."""

from dataclasses import dataclass, field
from typing import Any, Optional, List

PROJECT_STATUS_READY = "ready"
PROJECT_STATUS_PARTIAL = "partial"
PROJECT_STATUS_BLOCKED = "blocked"
PROJECT_STATUS_UNKNOWN = "unknown"
PROJECT_STATUS_KIND = "grafana-util-project-status"


import time
import datetime
from pathlib import Path

PROJECT_STATUS_FRESHNESS_CURRENT = "current"
PROJECT_STATUS_FRESHNESS_STALE = "stale"
PROJECT_STATUS_FRESHNESS_UNKNOWN = "unknown"
PROJECT_STATUS_STALE_AGE_SECONDS = 7 * 24 * 60 * 60


@dataclass
class ProjectStatusFreshnessSample:
    source: str
    age_seconds: Optional[int] = None
    observed_at_rfc3339: Optional[str] = None
    observed_at_timestamp: Optional[float] = None
    path: Optional[Path] = None

    @classmethod
    def from_age(cls, source: str, age_seconds: int) -> "ProjectStatusFreshnessSample":
        return cls(source=source, age_seconds=age_seconds)

    @classmethod
    def from_rfc3339(cls, source: str, observed_at: str) -> "ProjectStatusFreshnessSample":
        return cls(source=source, observed_at_rfc3339=observed_at)

    @classmethod
    def from_path(cls, source: str, path: Path) -> "ProjectStatusFreshnessSample":
        return cls(source=source, path=path)


def age_seconds_from_rfc3339(now: float, observed_at: str) -> Optional[int]:
    try:
        # Simplified RFC3339 parsing for common Grafana formats
        dt = datetime.datetime.fromisoformat(observed_at.replace("Z", "+00:00"))
        return max(0, int(now - dt.timestamp()))
    except Exception:
        return None


def freshness_from_ages(ages: List[int]) -> Optional[tuple[str, Optional[int], Optional[int]]]:
    if not ages:
        return None

    newest = min(ages)
    oldest = max(ages)
    status = PROJECT_STATUS_FRESHNESS_STALE if oldest > PROJECT_STATUS_STALE_AGE_SECONDS else PROJECT_STATUS_FRESHNESS_CURRENT
    return status, newest, oldest


def build_live_project_status_freshness(
    source_count: int,
    ages: List[int],
) -> ProjectStatusFreshness:
    effective_count = source_count if source_count > 0 else len(ages)
    if effective_count == 0:
        return ProjectStatusFreshness(status=PROJECT_STATUS_FRESHNESS_UNKNOWN, source_count=0)

    res = freshness_from_ages(ages)
    if res:
        status, newest, oldest = res
        return ProjectStatusFreshness(
            status=status,
            source_count=effective_count,
            newest_age_seconds=newest,
            oldest_age_seconds=oldest,
        )

    return ProjectStatusFreshness(status=PROJECT_STATUS_FRESHNESS_CURRENT, source_count=effective_count)


def build_live_project_status_freshness_from_samples(
    samples: List[ProjectStatusFreshnessSample],
) -> ProjectStatusFreshness:
    source_names = {s.source for s in samples}
    effective_count = len(source_names)
    if effective_count == 0:
        return ProjectStatusFreshness(status=PROJECT_STATUS_FRESHNESS_UNKNOWN, source_count=0)

    now = time.time()
    ages = []
    for s in samples:
        age = None
        if s.age_seconds is not None:
            age = s.age_seconds
        elif s.observed_at_rfc3339 is not None:
            age = age_seconds_from_rfc3339(now, s.observed_at_rfc3339)
        elif s.observed_at_timestamp is not None:
            age = max(0, int(now - s.observed_at_timestamp))
        elif s.path is not None and s.path.exists():
            age = max(0, int(now - s.path.stat().st_mtime))
        
        if age is not None:
            ages.append(age)

    res = freshness_from_ages(ages)
    if res:
        status, newest, oldest = res
        return ProjectStatusFreshness(
            status=status,
            source_count=effective_count,
            newest_age_seconds=newest,
            oldest_age_seconds=oldest,
        )

    return ProjectStatusFreshness(status=PROJECT_STATUS_FRESHNESS_CURRENT, source_count=effective_count)


def build_live_project_status_freshness_from_source_count(source_count: int) -> ProjectStatusFreshness:
    return build_live_project_status_freshness(source_count, [])


@dataclass
class ProjectStatusFreshness:
    status: str = ""
    source_count: int = 0
    newest_age_seconds: Optional[int] = None
    oldest_age_seconds: Optional[int] = None

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {
            "status": self.status,
            "sourceCount": self.source_count,
        }
        if self.newest_age_seconds is not None:
            result["newestAgeSeconds"] = self.newest_age_seconds
        if self.oldest_age_seconds is not None:
            result["oldestAgeSeconds"] = self.oldest_age_seconds
        return result


@dataclass
class ProjectStatusFinding:
    kind: str
    count: int
    source: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "count": self.count,
            "source": self.source,
        }


@dataclass
class ProjectDomainStatus:
    id: str
    scope: str
    mode: str
    status: str
    reason_code: str
    primary_count: int
    blocker_count: int
    warning_count: int
    source_kinds: List[str]
    signal_keys: List[str]
    blockers: List[ProjectStatusFinding]
    warnings: List[ProjectStatusFinding]
    next_actions: List[str]
    freshness: ProjectStatusFreshness

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "scope": self.scope,
            "mode": self.mode,
            "status": self.status,
            "reasonCode": self.reason_code,
            "primaryCount": self.primary_count,
            "blockerCount": self.blocker_count,
            "warningCount": self.warning_count,
            "sourceKinds": self.source_kinds,
            "signalKeys": self.signal_keys,
            "blockers": [b.to_dict() for b in self.blockers],
            "warnings": [w.to_dict() for w in self.warnings],
            "nextActions": self.next_actions,
            "freshness": self.freshness.to_dict(),
        }


@dataclass
class ProjectStatusOverall:
    status: str
    domain_count: int
    present_count: int
    blocked_count: int
    blocker_count: int
    warning_count: int
    freshness: ProjectStatusFreshness

    def to_dict(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "domainCount": self.domain_count,
            "presentCount": self.present_count,
            "blockedCount": self.blocked_count,
            "blockerCount": self.blocker_count,
            "warningCount": self.warning_count,
            "freshness": self.freshness.to_dict(),
        }


@dataclass
class ProjectStatusRankedFinding:
    domain: str
    kind: str
    count: int
    source: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "domain": self.domain,
            "kind": self.kind,
            "count": self.count,
            "source": self.source,
        }


@dataclass
class ProjectStatusAction:
    domain: str
    reason_code: str
    action: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "domain": self.domain,
            "reasonCode": self.reason_code,
            "action": self.action,
        }


@dataclass
class ProjectStatus:
    schema_version: int
    tool_version: str
    scope: str
    overall: ProjectStatusOverall
    domains: List[ProjectDomainStatus]
    top_blockers: List[ProjectStatusRankedFinding]
    next_actions: List[ProjectStatusAction]
    discovery: Optional[dict[str, Any]] = None

    def to_dict(self) -> dict[str, Any]:
        result = {
            "kind": PROJECT_STATUS_KIND,
            "schemaVersion": self.schema_version,
            "toolVersion": self.tool_version,
            "scope": self.scope,
            "overall": self.overall.to_dict(),
            "domains": [d.to_dict() for d in self.domains],
            "topBlockers": [b.to_dict() for b in self.top_blockers],
            "nextActions": [a.to_dict() for a in self.next_actions],
        }
        if self.discovery is not None:
            result["discovery"] = self.discovery
        return result


def status_finding(kind: str, count: int, source: str) -> ProjectStatusFinding:
    return ProjectStatusFinding(kind=kind, count=count, source=source)


def render_domain_finding_summary(findings: List[ProjectStatusFinding]) -> Optional[str]:
    if not findings:
        return None
    return ",".join(f"{f.kind}:{f.count}" for f in findings)


def render_project_status_decision_order(status: ProjectStatus) -> Optional[List[str]]:
    ordered_domains = []
    for blocker in status.top_blockers:
        if blocker.domain not in ordered_domains:
            ordered_domains.append(blocker.domain)
    for action in status.next_actions:
        if action.domain not in ordered_domains:
            ordered_domains.append(action.domain)
    
    if not ordered_domains:
        return None

    lines = []
    for index, domain_id in enumerate(ordered_domains):
        blockers = [f"{b.kind}:{b.count}" for b in status.top_blockers if b.domain == domain_id]
        actions = [a.action for a in status.next_actions if a.domain == domain_id]
        line = f"{index + 1}. {domain_id}"
        if blockers:
            line += f" blockers={', '.join(blockers)}"
        if actions:
            line += f" next={' / '.join(actions)}"
        lines.append(line)
    return lines


def render_project_status_signal_summary(status: ProjectStatus) -> Optional[str]:
    fragments = []
    for domain in status.domains:
        has_sync_signals = any(k in ("sync-summary", "bundle-preflight", "promotion-preflight") for k in domain.source_kinds)
        should_render = has_sync_signals or len(domain.source_kinds) > 1
        if not should_render or not domain.source_kinds:
            continue
        fragments.append(
            f"{domain.id} sources={','.join(domain.source_kinds)} signalKeys={len(domain.signal_keys)} blockers={domain.blocker_count} warnings={domain.warning_count}"
        )
    
    if not fragments:
        return None
    return f"Signals: {'; '.join(fragments)}"


def build_project_top_blockers(
    domains: List[ProjectDomainStatus],
) -> List[ProjectStatusRankedFinding]:
    blockers = []
    for domain in domains:
        for blocker in domain.blockers:
            blockers.append(
                ProjectStatusRankedFinding(
                    domain=domain.id,
                    kind=blocker.kind,
                    count=blocker.count,
                    source=blocker.source,
                )
            )
    # Sort by count descending, then domain, then kind, then source
    blockers.sort(key=lambda x: (-x.count, x.domain, x.kind, x.source))
    return blockers


def domain_status_rank(status: str) -> int:
    if status == PROJECT_STATUS_BLOCKED:
        return 0
    if status == PROJECT_STATUS_PARTIAL:
        return 1
    if status == PROJECT_STATUS_READY:
        return 2
    return 3


def build_project_next_actions(
    domains: List[ProjectDomainStatus],
) -> List[ProjectStatusAction]:
    actions = []
    for domain in domains:
        for action in domain.next_actions:
            actions.append(
                ProjectStatusAction(
                    domain=domain.id,
                    reason_code=domain.reason_code,
                    action=action,
                )
            )

    domain_map = {d.id: d for d in domains}

    def sort_key(item: ProjectStatusAction):
        domain = domain_map[item.domain]
        return (
            domain_status_rank(domain.status),
            -domain.blocker_count,
            -domain.warning_count,
            item.domain,
            item.action,
        )

    actions.sort(key=sort_key)
    return actions


def build_project_status(
    scope: str,
    domain_count: int,
    freshness: ProjectStatusFreshness,
    domains: List[ProjectDomainStatus],
    tool_version: str = "0.0.0",
) -> ProjectStatus:
    present_count = len(domains)
    blocked_count = sum(1 for d in domains if d.status == PROJECT_STATUS_BLOCKED)
    blocker_count = sum(d.blocker_count for d in domains)
    warning_count = sum(d.warning_count for d in domains)

    if blocked_count > 0:
        overall_status = PROJECT_STATUS_BLOCKED
    elif present_count < domain_count or any(
        d.status in (PROJECT_STATUS_PARTIAL, PROJECT_STATUS_UNKNOWN) for d in domains
    ):
        overall_status = PROJECT_STATUS_PARTIAL
    else:
        overall_status = PROJECT_STATUS_READY

    overall = ProjectStatusOverall(
        status=overall_status,
        domain_count=domain_count,
        present_count=present_count,
        blocked_count=blocked_count,
        blocker_count=blocker_count,
        warning_count=warning_count,
        freshness=freshness,
    )

    return ProjectStatus(
        schema_version=1,
        tool_version=tool_version,
        scope=scope,
        overall=overall,
        domains=domains,
        top_blockers=build_project_top_blockers(domains),
        next_actions=build_project_next_actions(domains),
    )
