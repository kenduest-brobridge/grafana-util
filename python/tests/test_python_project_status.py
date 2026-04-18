import unittest
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


if __name__ == "__main__":
    unittest.main()
