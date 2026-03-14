"""Shared datasource import/export normalization helpers."""

from typing import Any, Dict


def normalize_datasource_string(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, bool):
        return "true" if value else "false"
    return str(value).strip()


def normalize_datasource_bool(value: Any) -> bool:
    normalized = normalize_datasource_string(value).lower()
    return normalized in ("true", "1", "yes")


def normalize_datasource_record(record: Dict[str, Any]) -> Dict[str, str]:
    return {
        "uid": normalize_datasource_string(record.get("uid")),
        "name": normalize_datasource_string(record.get("name")),
        "type": normalize_datasource_string(record.get("type")),
        "access": normalize_datasource_string(record.get("access")),
        "url": normalize_datasource_string(record.get("url")),
        "isDefault": "true"
        if normalize_datasource_bool(record.get("isDefault"))
        else "false",
        "org": normalize_datasource_string(record.get("org")),
        "orgId": normalize_datasource_string(record.get("orgId")),
    }
