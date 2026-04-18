"""Shared review/action contract vocabulary."""

REVIEW_ACTION_BLOCKED = "blocked"
REVIEW_ACTION_BLOCKED_AMBIGUOUS = "blocked-ambiguous"
REVIEW_ACTION_BLOCKED_MISSING_ORG = "blocked-missing-org"
REVIEW_ACTION_BLOCKED_READ_ONLY = "blocked-read-only"
REVIEW_ACTION_BLOCKED_TARGET = "blocked-target"
REVIEW_ACTION_BLOCKED_UID_MISMATCH = "blocked-uid-mismatch"
REVIEW_ACTION_EXTRA_REMOTE = "extra-remote"
REVIEW_ACTION_SAME = "same"
REVIEW_ACTION_UNMANAGED = "unmanaged"
REVIEW_ACTION_WOULD_CREATE = "would-create"
REVIEW_ACTION_WOULD_DELETE = "would-delete"
REVIEW_ACTION_WOULD_UPDATE = "would-update"

REVIEW_STATUS_BLOCKED = "blocked"
REVIEW_STATUS_READY = "ready"
REVIEW_STATUS_SAME = "same"
REVIEW_STATUS_WARNING = "warning"

REVIEW_REASON_AMBIGUOUS_LIVE_NAME_MATCH = "ambiguous-live-name-match"
REVIEW_REASON_TARGET_ORG_MISSING = "target-org-missing"
REVIEW_REASON_TARGET_PROVISIONED_OR_MANAGED = "target-provisioned-or-managed"
REVIEW_REASON_TARGET_READ_ONLY = "target-read-only"
REVIEW_REASON_UID_NAME_MISMATCH = "uid-name-mismatch"

REVIEW_HINT_MISSING_REMOTE = "missing-remote"
REVIEW_HINT_REMOTE_ONLY = "remote-only"
REVIEW_HINT_REQUIRES_SECRET_VALUES = "requires-secret-values"


def is_review_apply_action(action: str) -> bool:
    return action in (
        REVIEW_ACTION_WOULD_CREATE,
        REVIEW_ACTION_WOULD_UPDATE,
        REVIEW_ACTION_WOULD_DELETE,
    )


def is_review_blocked_action(action: str) -> bool:
    return (
        action.startswith("blocked-")
        or action == REVIEW_ACTION_BLOCKED
        or action == REVIEW_ACTION_UNMANAGED
    )
