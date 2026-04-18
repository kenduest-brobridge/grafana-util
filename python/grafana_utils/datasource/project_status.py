"""Datasource live domain-status producer."""

from typing import Any, Optional, List, Set, Tuple, Dict
from ..project_status import (
    status_finding,
    ProjectDomainStatus,
    PROJECT_STATUS_PARTIAL,
    PROJECT_STATUS_READY,
)
from ..status_model import StatusReading, StatusRecordCount


DATASOURCE_DOMAIN_ID = "datasource"
DATASOURCE_SCOPE = "live"
DATASOURCE_MODE = "live-inventory"
DATASOURCE_REASON_READY = PROJECT_STATUS_READY
DATASOURCE_REASON_PARTIAL_NO_DATA = "partial-no-data"

DATASOURCE_SOURCE_KIND_LIST = "live-datasource-list"
DATASOURCE_SOURCE_KIND_READ = "live-datasource-read"
DATASOURCE_SOURCE_KIND_ORG_LIST = "live-org-list"
DATASOURCE_SOURCE_KIND_ORG_READ = "live-org-read"

DATASOURCE_SIGNAL_KEYS = [
    "live.datasourceCount",
    "live.defaultCount",
    "live.orgCount",
    "live.orgIdCount",
    "live.uidCount",
    "live.nameCount",
    "live.accessCount",
    "live.typeCount",
    "live.jsonDataCount",
    "live.basicAuthCount",
    "live.basicAuthPasswordCount",
    "live.passwordCount",
    "live.httpHeaderValueCount",
    "live.withCredentialsCount",
    "live.secureJsonFieldsCount",
    "live.tlsAuthCount",
    "live.tlsSkipVerifyCount",
    "live.serverNameCount",
    "live.readOnlyCount",
]

DATASOURCE_WARNING_MISSING_DEFAULT = "missing-default"
DATASOURCE_WARNING_MULTIPLE_DEFAULTS = "multiple-defaults"
DATASOURCE_WARNING_MISSING_UID = "missing-uid"
DATASOURCE_WARNING_DUPLICATE_UID = "duplicate-uid"
DATASOURCE_WARNING_MISSING_NAME = "missing-name"
DATASOURCE_WARNING_MISSING_ACCESS = "missing-access"
DATASOURCE_WARNING_MISSING_TYPE = "missing-type"
DATASOURCE_WARNING_MISSING_ORG_ID = "missing-org-id"
DATASOURCE_WARNING_MIXED_ORG_IDS = "mixed-org-ids"
DATASOURCE_WARNING_ORG_SCOPE_MISMATCH = "org-scope-mismatch"
DATASOURCE_WARNING_ORG_LIST_MISMATCH = "org-list-mismatch"
DATASOURCE_WARNING_PROVIDER_JSON_DATA = "provider-json-data-present"
DATASOURCE_WARNING_BASIC_AUTH = "basic-auth-configured"
DATASOURCE_WARNING_BASIC_AUTH_PASSWORD = "basic-auth-password-present"
DATASOURCE_WARNING_PASSWORD = "datasource-password-present"
DATASOURCE_WARNING_HTTP_HEADER_VALUES = "http-header-secret-values-present"
DATASOURCE_WARNING_WITH_CREDENTIALS = "with-credentials-configured"
DATASOURCE_WARNING_SECURE_JSON_FIELDS = "secure-json-fields-present"
DATASOURCE_WARNING_TLS_AUTH = "tls-auth-configured"
DATASOURCE_WARNING_TLS_SKIP_VERIFY = "tls-skip-verify-configured"
DATASOURCE_WARNING_SERVER_NAME = "server-name-configured"
DATASOURCE_WARNING_READ_ONLY = "read-only"

DATASOURCE_CREATE_OR_SYNC_ACTIONS = ["create or sync at least one datasource in Grafana"]
DATASOURCE_MARK_DEFAULT_ACTIONS = ["mark a default datasource in Grafana"]
DATASOURCE_KEEP_SINGLE_DEFAULT_ACTIONS = [
    "keep exactly one datasource marked as the default"
]
DATASOURCE_FIX_ORG_SCOPE_ACTIONS = [
    "re-run live datasource read after aligning datasource org scope with the current org and visible org list",
]
DATASOURCE_FIX_METADATA_ACTIONS = [
    "re-run live datasource read after correcting datasource identity or org scope"
]
DATASOURCE_REVIEW_SECRET_PROVIDER_ACTIONS = [
    "review live datasource secret and provider fields before export or import"
]


def record_bool(record: Dict[str, Any], key: str) -> bool:
    val = record.get(key)
    return bool(val) if isinstance(val, bool) else False


def record_scalar(record: Dict[str, Any], key: str) -> str:
    val = record.get(key)
    if val is None:
        return ""
    if isinstance(val, (str, int, float, bool)):
        return str(val)
    return ""


def nested_object(record: Dict[str, Any], key: str) -> Optional[Dict[str, Any]]:
    val = record.get(key)
    return val if isinstance(val, dict) else None


def nested_record_bool(record: Dict[str, Any], parent_key: str, child_key: str) -> bool:
    obj = nested_object(record, parent_key)
    if obj:
        val = obj.get(child_key)
        return bool(val) if isinstance(val, bool) else False
    return False


def nested_record_string(record: Dict[str, Any], parent_key: str, child_key: str) -> str:
    obj = nested_object(record, parent_key)
    if obj:
        val = obj.get(child_key)
        if isinstance(val, str):
            val = val.strip()
            return val if val else ""
    return ""


def distinct_non_empty_values(records: List[Dict[str, Any]], key: str) -> int:
    values: Set[str] = set()
    for record in records:
        val = record_scalar(record, key)
        if val:
            values.add(val)
    return len(values)


def missing_scalar_values(records: List[Dict[str, Any]], key: str) -> int:
    count = 0
    for record in records:
        if not record_scalar(record, key):
            count += 1
    return count


def non_empty_object_values(records: List[Dict[str, Any]], key: str) -> int:
    count = 0
    for record in records:
        obj = nested_object(record, key)
        if obj and obj:
            count += 1
    return count


def nested_bool_values(records: List[Dict[str, Any]], parent_key: str, child_key: str) -> int:
    count = 0
    for record in records:
        if nested_record_bool(record, parent_key, child_key):
            count += 1
    return count


def nested_string_values(records: List[Dict[str, Any]], parent_key: str, child_key: str) -> int:
    count = 0
    for record in records:
        if nested_record_string(record, parent_key, child_key):
            count += 1
    return count


def nested_bool_key_prefix_values(
    records: List[Dict[str, Any]], parent_key: str, child_key_prefix: str
) -> int:
    total = 0
    for record in records:
        obj = nested_object(record, parent_key)
        if obj:
            for key, val in obj.items():
                if key.startswith(child_key_prefix) and isinstance(val, bool) and val:
                    total += 1
    return total


DATASOURCE_REEXPORT_AFTER_CHANGES_ACTIONS = ["re-run datasource export after datasource changes"]


def build_datasource_project_status(
    summary_document: Optional[Dict[str, Any]] = None,
) -> Optional[ProjectDomainStatus]:
    if summary_document is None:
        return None

    summary = summary_document.get("summary", {})
    datasource_count = int(summary.get("datasourceCount", 0))
    default_count = int(summary.get("defaultCount", 0))

    is_partial = datasource_count == 0

    if is_partial:
        status = PROJECT_STATUS_PARTIAL
        reason_code = DATASOURCE_REASON_PARTIAL_NO_DATA
        next_actions = list(DATASOURCE_CREATE_OR_SYNC_ACTIONS)
    else:
        status = PROJECT_STATUS_READY
        reason_code = DATASOURCE_REASON_READY
        next_actions = list(DATASOURCE_REEXPORT_AFTER_CHANGES_ACTIONS)

    warnings = []
    if datasource_count > 0 and default_count == 0:
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_DEFAULT, 1, "summary.defaultCount"))
        next_actions.extend(DATASOURCE_MARK_DEFAULT_ACTIONS)
    elif default_count > 1:
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MULTIPLE_DEFAULTS, default_count - 1, "summary.defaultCount"))
        next_actions.extend(DATASOURCE_KEEP_SINGLE_DEFAULT_ACTIONS)

    return StatusReading(
        id=DATASOURCE_DOMAIN_ID,
        scope="staged",
        mode="artifact-summary",
        status=status,
        reason_code=reason_code,
        primary_count=datasource_count,
        source_kinds=["datasource-export"],
        signal_keys=["summary.datasourceCount", "summary.defaultCount"],
        blockers=[],
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()


def build_datasource_live_project_status(
    datasource_list: Optional[List[Dict[str, Any]]] = None,
    datasource_read: Optional[Dict[str, Any]] = None,
    org_list: Optional[List[Dict[str, Any]]] = None,
    current_org: Optional[Dict[str, Any]] = None,
) -> Optional[ProjectDomainStatus]:
    if datasource_list is None and datasource_read is None:
        return None

    if datasource_list is not None:
        records = datasource_list
        datasource_source_kind = DATASOURCE_SOURCE_KIND_LIST
    else:
        records = [datasource_read]
        datasource_source_kind = DATASOURCE_SOURCE_KIND_READ

    datasource_count = len(records)
    default_count = sum(1 for r in records if record_bool(r, "isDefault"))
    uid_count = distinct_non_empty_values(records, "uid")
    org_id_count = distinct_non_empty_values(records, "orgId")
    missing_uid_count = missing_scalar_values(records, "uid")
    missing_name_count = missing_scalar_values(records, "name")
    missing_access_count = missing_scalar_values(records, "access")
    missing_type_count = missing_scalar_values(records, "type")
    missing_org_id_count = missing_scalar_values(records, "orgId")

    basic_auth_count = sum(1 for r in records if record_bool(r, "basicAuth"))
    basic_auth_password_count = nested_bool_values(records, "secureJsonFields", "basicAuthPassword")
    password_count = nested_bool_values(records, "secureJsonFields", "password")
    http_header_value_count = nested_bool_key_prefix_values(records, "secureJsonFields", "httpHeaderValue")
    with_credentials_count = sum(1 for r in records if record_bool(r, "withCredentials"))
    json_data_count = non_empty_object_values(records, "jsonData")
    secure_json_fields_count = non_empty_object_values(records, "secureJsonFields")
    tls_auth_count = nested_bool_values(records, "jsonData", "tlsAuth")
    tls_skip_verify_count = nested_bool_values(records, "jsonData", "tlsSkipVerify")
    server_name_count = nested_string_values(records, "jsonData", "serverName")
    read_only_count = sum(1 for r in records if record_bool(r, "readOnly"))

    source_kinds = [datasource_source_kind]
    if org_list is not None:
        source_kinds.append(DATASOURCE_SOURCE_KIND_ORG_LIST)
    elif current_org is not None:
        source_kinds.append(DATASOURCE_SOURCE_KIND_ORG_READ)

    warnings = []
    metadata_issue_found = False
    org_scope_issue_found = False
    readiness_signal_found = False

    if missing_uid_count > 0:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_UID, missing_uid_count, "live.uidCount"))
    if missing_name_count > 0:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_NAME, missing_name_count, "live.nameCount"))
    if missing_access_count > 0:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_ACCESS, missing_access_count, "live.accessCount"))
    if datasource_count > 0 and uid_count + missing_uid_count < datasource_count:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_DUPLICATE_UID, datasource_count - uid_count - missing_uid_count, "live.uidCount"))
    if missing_type_count > 0:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_TYPE, missing_type_count, "live.typeCount"))
    if missing_org_id_count > 0:
        metadata_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_ORG_ID, missing_org_id_count, "live.orgIdCount"))

    if current_org is not None and org_id_count > 1:
        metadata_issue_found = True
        org_scope_issue_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MIXED_ORG_IDS, org_id_count - 1, "live.orgIdCount"))

    current_org_id = record_scalar(current_org, "id") if current_org else ""
    if current_org_id and org_id_count == 1:
        datasource_org_id = next((record_scalar(r, "orgId") for r in records if record_scalar(r, "orgId")), "")
        if datasource_org_id and datasource_org_id != current_org_id:
            metadata_issue_found = True
            org_scope_issue_found = True
            warnings.append(StatusRecordCount(DATASOURCE_WARNING_ORG_SCOPE_MISMATCH, 1, "live.orgIdCount"))

    if org_list:
        org_list_ids = {record_scalar(o, "id") for o in org_list if record_scalar(o, "id")}
        datasource_org_ids = {record_scalar(r, "orgId") for r in records if record_scalar(r, "orgId")}
        missing_org_ids = len(datasource_org_ids - org_list_ids)
        if missing_org_ids > 0:
            metadata_issue_found = True
            org_scope_issue_found = True
            warnings.append(StatusRecordCount(DATASOURCE_WARNING_ORG_LIST_MISMATCH, missing_org_ids, "live.orgCount"))

    if json_data_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_PROVIDER_JSON_DATA, json_data_count, "live.jsonDataCount"))
    if basic_auth_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_BASIC_AUTH, basic_auth_count, "live.basicAuthCount"))
    if basic_auth_password_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_BASIC_AUTH_PASSWORD, basic_auth_password_count, "live.basicAuthPasswordCount"))
    if password_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_PASSWORD, password_count, "live.passwordCount"))
    if http_header_value_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_HTTP_HEADER_VALUES, http_header_value_count, "live.httpHeaderValueCount"))
    if with_credentials_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_WITH_CREDENTIALS, with_credentials_count, "live.withCredentialsCount"))
    if secure_json_fields_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_SECURE_JSON_FIELDS, secure_json_fields_count, "live.secureJsonFieldsCount"))
    if tls_auth_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_TLS_AUTH, tls_auth_count, "live.tlsAuthCount"))
    if tls_skip_verify_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_TLS_SKIP_VERIFY, tls_skip_verify_count, "live.tlsSkipVerifyCount"))
    if server_name_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_SERVER_NAME, server_name_count, "live.serverNameCount"))
    if read_only_count > 0:
        readiness_signal_found = True
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_READ_ONLY, read_only_count, "live.readOnlyCount"))

    next_actions = []
    if datasource_count == 0:
        next_actions.extend(DATASOURCE_CREATE_OR_SYNC_ACTIONS)
    elif default_count == 0:
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MISSING_DEFAULT, 1, "live.defaultCount"))
        next_actions.extend(DATASOURCE_MARK_DEFAULT_ACTIONS)
    elif default_count > 1:
        warnings.append(StatusRecordCount(DATASOURCE_WARNING_MULTIPLE_DEFAULTS, default_count - 1, "live.defaultCount"))
        next_actions.extend(DATASOURCE_KEEP_SINGLE_DEFAULT_ACTIONS)

    if org_scope_issue_found and datasource_count > 0:
        next_actions.extend(DATASOURCE_FIX_ORG_SCOPE_ACTIONS)
    elif metadata_issue_found and datasource_count > 0:
        next_actions.extend(DATASOURCE_FIX_METADATA_ACTIONS)
    if readiness_signal_found and datasource_count > 0:
        next_actions.extend(DATASOURCE_REVIEW_SECRET_PROVIDER_ACTIONS)

    status = PROJECT_STATUS_PARTIAL if datasource_count == 0 else PROJECT_STATUS_READY
    reason_code = DATASOURCE_REASON_PARTIAL_NO_DATA if datasource_count == 0 else DATASOURCE_REASON_READY

    return StatusReading(
        id=DATASOURCE_DOMAIN_ID,
        scope=DATASOURCE_SCOPE,
        mode=DATASOURCE_MODE,
        status=status,
        reason_code=reason_code,
        primary_count=datasource_count,
        source_kinds=source_kinds,
        signal_keys=DATASOURCE_SIGNAL_KEYS,
        blockers=[],
        warnings=warnings,
        next_actions=next_actions,
    ).into_project_domain_status()
