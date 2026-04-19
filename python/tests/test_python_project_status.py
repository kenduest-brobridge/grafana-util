import unittest
from grafana_utils import project_status_live
from grafana_utils.project_status import (
    build_project_status,
    ProjectStatusFreshness,
    ProjectDomainStatus,
    PROJECT_STATUS_READY,
    PROJECT_STATUS_PARTIAL,
)
from grafana_utils.status_model import StatusReading


class TestProjectStatus(unittest.TestCase):
    def test_build_project_status_ready(self):
        domain = StatusReading(
            id="test",
            scope="live",
            mode="test-mode",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=1,
            source_kinds=["test-source"],
            signal_keys=["test.count"],
            blockers=[],
            warnings=[],
            next_actions=["test-action"],
        ).into_project_domain_status()

        status = build_project_status(
            scope="live",
            domain_count=1,
            freshness=ProjectStatusFreshness(status="ready", source_count=1),
            domains=[domain],
        )

        self.assertEqual(status.overall.status, PROJECT_STATUS_READY)
        self.assertEqual(status.overall.domain_count, 1)
        self.assertEqual(status.overall.present_count, 1)
        self.assertEqual(status.overall.blocked_count, 0)

    def test_build_project_status_partial(self):
        status = build_project_status(
            scope="live",
            domain_count=2,
            freshness=ProjectStatusFreshness(status="ready", source_count=0),
            domains=[],
        )

        self.assertEqual(status.overall.status, PROJECT_STATUS_PARTIAL)
        self.assertEqual(status.overall.domain_count, 2)
        self.assertEqual(status.overall.present_count, 0)

    def test_live_read_failed_domain_is_blocked(self):
        domain = project_status_live._read_failed_domain(
            "dashboard",
            source="live.dashboard",
            error=RuntimeError("boom"),
            next_action="restore dashboard access",
        )

        self.assertEqual(domain.id, "dashboard")
        self.assertEqual(domain.status, "blocked")
        self.assertEqual(domain.reason_code, "live-read-failed")
        self.assertEqual(domain.blocker_count, 1)
        self.assertIn("restore dashboard access", domain.next_actions)

    def test_merge_live_domain_statuses_adds_all_org_mode_suffix(self):
        first = StatusReading(
            id="dashboard",
            scope="live",
            mode="live-list-surfaces",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=2,
            source_kinds=["live-dashboard-search"],
            signal_keys=["live.dashboards.count"],
            blockers=[],
            warnings=[],
            next_actions=["review dashboards"],
            freshness=ProjectStatusFreshness(status="current", source_count=1),
        ).into_project_domain_status()
        second = StatusReading(
            id="dashboard",
            scope="live",
            mode="live-list-surfaces",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=3,
            source_kinds=["live-dashboard-search"],
            signal_keys=["live.dashboards.count"],
            blockers=[],
            warnings=[],
            next_actions=["review dashboards"],
            freshness=ProjectStatusFreshness(status="current", source_count=1),
        ).into_project_domain_status()

        merged = project_status_live.merge_live_domain_statuses([first, second])

        self.assertEqual(merged.primary_count, 5)
        self.assertEqual(merged.mode, "live-list-surfaces-all-orgs")
        self.assertEqual(merged.freshness.source_count, 2)


if __name__ == "__main__":
    unittest.main()
