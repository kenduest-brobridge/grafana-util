"""Repo-local grafana-util profile configuration helpers."""

from __future__ import annotations

import base64
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Optional

from . import yaml_compat as yaml

DEFAULT_CONFIG_FILENAME = "grafana-util.yaml"
PROFILE_CONFIG_ENV_VAR = "GRAFANA_UTIL_CONFIG"
DEFAULT_SECRETS_DIRNAME = ".grafana-util-secrets"
DEFAULT_PROFILE_VERSION = 1
DEFAULT_PROFILE_SCOPE_NAME = "default"
LATEST_RUN_POINTER_FILENAME = "latest-run.json"


@dataclass(frozen=True)
class ProfileConnectionDetails:
    """Resolved connection details for one selected profile."""

    url: str
    headers: dict[str, str]
    timeout: int
    verify_ssl: bool
    insecure: bool
    ca_cert: Optional[str]
    org_id: Optional[str]
    profile_name: Optional[str]


def resolve_config_path(config_path: Optional[Path | str] = None) -> Path:
    """Return the repo-local profile config path."""
    if config_path is None:
        configured = os.environ.get(PROFILE_CONFIG_ENV_VAR)
        if configured:
            return Path(configured)
        return Path(DEFAULT_CONFIG_FILENAME)
    return Path(config_path)


def default_artifact_root_path_for_config_path(config_path: Path | str) -> Path:
    """Return the default artifact root beside one profile config file."""
    path = Path(config_path)
    config_dir = path.parent if str(path.parent) else Path(".")
    return config_dir / ".grafana-util" / "artifacts"


def resolve_artifact_root_path(
    document: Optional[Mapping[str, Any]] = None,
    config_path: Optional[Path | str] = None,
) -> Path:
    """Resolve artifact_root using the same config-relative rules as Rust."""
    resolved_config_path = resolve_config_path(config_path)
    default_root = default_artifact_root_path_for_config_path(resolved_config_path)
    if not document:
        return default_root
    raw_root = document.get("artifact_root")
    if raw_root is None:
        return default_root
    root = Path(str(raw_root))
    if root.is_absolute():
        return root
    config_dir = resolved_config_path.parent if str(resolved_config_path.parent) else Path(".")
    return config_dir / root


def profile_scope_name(profile_name: Optional[str] = None) -> str:
    """Return the artifact workspace profile scope name."""
    return _normalize_text(profile_name, DEFAULT_PROFILE_SCOPE_NAME)


def profile_scope_path(artifact_root: Path | str, profile_name: Optional[str] = None) -> Path:
    """Return the artifact workspace scope root for one profile."""
    return Path(artifact_root) / profile_scope_name(profile_name)


def _validate_run_id(run_id: str) -> None:
    if not run_id:
        raise ValueError("Run id must not be empty.")
    if run_id in {".", ".."}:
        raise ValueError("Run id must not be '.' or '..'.")
    if "/" in run_id or "\\" in run_id:
        raise ValueError("Run id must not contain path separators.")


def read_latest_run_id(scope_root: Path | str) -> str:
    """Read the latest artifact run id from <scope>/latest-run.json."""
    pointer_path = Path(scope_root) / LATEST_RUN_POINTER_FILENAME
    if not pointer_path.is_file():
        raise ValueError(f"No latest run recorded at {pointer_path}.")
    raw = yaml.safe_load(pointer_path.read_text(encoding="utf-8"))
    if not isinstance(raw, Mapping):
        raise ValueError(f"Latest run pointer must be a JSON/YAML object: {pointer_path}")
    run_id = _normalize_text(raw.get("run_id") or raw.get("runId"))
    _validate_run_id(run_id)
    return run_id


def resolve_artifact_run_id(scope_root: Path | str, run: Optional[str], run_id: Optional[str]) -> str:
    """Resolve a local artifact run selector for read-only consumers."""
    if run_id:
        selected = _normalize_text(run_id)
        _validate_run_id(selected)
        return selected
    selector = _normalize_text(run, "latest")
    if selector == "latest":
        return read_latest_run_id(scope_root)
    if selector == "timestamp":
        runs_root = Path(scope_root) / "runs"
        candidates = sorted(
            path.name
            for path in runs_root.iterdir()
            if path.is_dir() and path.name not in {".", ".."}
        ) if runs_root.is_dir() else []
        if not candidates:
            raise ValueError(f"No timestamp run recorded under {runs_root}.")
        selected = candidates[-1]
        _validate_run_id(selected)
        return selected
    raise ValueError(f"Unsupported artifact run selector '{selector}'. Use latest or timestamp.")


def resolve_input_lane_path(
    lane_path: str,
    *,
    run: Optional[str] = None,
    run_id: Optional[str] = None,
    profile_name: Optional[str] = None,
    config_path: Optional[Path | str] = None,
) -> Path:
    """Resolve a read-only artifact lane path from profile config workspace settings."""
    document = load_profile_document(config_path)
    scope_root = profile_scope_path(
        resolve_artifact_root_path(document, config_path),
        profile_name,
    )
    selected_run_id = resolve_artifact_run_id(scope_root, run, run_id)
    return scope_root / "runs" / selected_run_id / lane_path


def load_profile_document(config_path: Optional[Path | str] = None) -> dict[str, Any]:
    """Load one profile config document or return the default empty shape."""
    path = resolve_config_path(config_path)
    if not path.is_file():
        return {
            "version": DEFAULT_PROFILE_VERSION,
            "defaultProfile": None,
            "profiles": {},
        }
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"Profile config must be a YAML object: {path}")
    data.setdefault("version", DEFAULT_PROFILE_VERSION)
    data.setdefault("defaultProfile", None)
    data.setdefault("profiles", {})
    if not isinstance(data["profiles"], dict):
        raise ValueError(f"Profile config profiles must be a YAML object: {path}")
    return data


def save_profile_document(
    document: Mapping[str, Any],
    config_path: Optional[Path | str] = None,
) -> Path:
    """Write one profile config document to disk."""
    path = resolve_config_path(config_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(dict(document)),
        encoding="utf-8",
    )
    return path


def _normalize_text(value: Any, default: str = "") -> str:
    text = str(value or "").strip()
    return text or default


def _mask(value: Any) -> str:
    return "******" if _normalize_text(value) else ""


def _encode_secret(value: str) -> str:
    return base64.b64encode(value.encode("utf-8")).decode("ascii")


def _decode_secret(value: str) -> str:
    try:
        return base64.b64decode(value.encode("ascii")).decode("utf-8")
    except Exception as exc:  # pragma: no cover - defensive
        raise ValueError(f"Invalid encoded profile secret: {exc}") from exc


def list_profile_names(document: Mapping[str, Any]) -> list[str]:
    profiles = document.get("profiles") or {}
    if not isinstance(profiles, Mapping):
        return []
    return sorted(str(name) for name in profiles.keys())


def select_profile(
    document: Mapping[str, Any],
    profile_name: Optional[str] = None,
) -> tuple[str, dict[str, Any]]:
    """Select one profile by name or default selection rules."""
    profiles = document.get("profiles") or {}
    if not isinstance(profiles, Mapping):
        raise ValueError("Profile config profiles must be a mapping.")
    selected_name = _normalize_text(
        profile_name or document.get("defaultProfile") or next(iter(profiles), None)
    )
    if not selected_name:
        raise ValueError("No profile is available in grafana-util.yaml.")
    profile = profiles.get(selected_name)
    if profile is None:
        raise ValueError(f"Profile not found: {selected_name}")
    if not isinstance(profile, Mapping):
        raise ValueError(f"Profile {selected_name} must be a mapping.")
    return selected_name, dict(profile)


def _profile_secret_value(profile: Mapping[str, Any], key: str) -> Optional[str]:
    value = profile.get(key)
    if value is None:
        encoded = profile.get(f"{key}Encoded")
        if encoded is not None:
            return _decode_secret(str(encoded))
        return None
    return _normalize_text(value) or None


def build_profile_summary(
    name: str,
    profile: Mapping[str, Any],
    show_secrets: bool = False,
) -> dict[str, Any]:
    """Normalize one profile into a reviewable summary document."""
    auth = profile.get("auth") or {}
    if not isinstance(auth, Mapping):
        auth = {}
    secret_mode = _normalize_text(auth.get("secretMode"), "file")
    token = _profile_secret_value(auth, "apiToken") if show_secrets else _mask(
        auth.get("apiToken") or auth.get("apiTokenEncoded")
    )
    basic_password = _profile_secret_value(auth, "basicPassword") if show_secrets else _mask(
        auth.get("basicPassword") or auth.get("basicPasswordEncoded")
    )
    return {
        "name": name,
        "url": _normalize_text(profile.get("url")),
        "orgId": _normalize_text(profile.get("orgId")),
        "timeout": profile.get("timeout"),
        "verifySsl": bool(profile.get("verifySsl", False)),
        "insecure": bool(profile.get("insecure", False)),
        "caCert": _normalize_text(profile.get("caCert")),
        "auth": {
            "mode": _normalize_text(auth.get("mode"), "none"),
            "basicUser": _normalize_text(auth.get("basicUser")),
            "apiToken": token if show_secrets else _mask(token),
            "basicPassword": basic_password if show_secrets else _mask(basic_password),
            "tokenEnv": _normalize_text(auth.get("tokenEnv")),
            "passwordEnv": _normalize_text(auth.get("passwordEnv")),
            "secretMode": secret_mode,
            "secretPassphraseEnv": _normalize_text(auth.get("secretPassphraseEnv")),
        },
    }


def render_profile_summary_text(summary: Mapping[str, Any]) -> list[str]:
    """Render one profile summary as plain text."""
    auth = summary.get("auth") or {}
    lines = [
        f"Profile: {summary.get('name') or 'unknown'}",
        f"URL: {summary.get('url') or ''}",
        f"Org ID: {summary.get('orgId') or ''}",
        f"Timeout: {summary.get('timeout') or ''}",
        f"verifySsl: {bool(summary.get('verifySsl'))}",
        f"insecure: {bool(summary.get('insecure'))}",
        f"CA cert: {summary.get('caCert') or ''}",
        f"Auth mode: {auth.get('mode') or 'none'}",
        f"Basic user: {auth.get('basicUser') or ''}",
        f"API token: {auth.get('apiToken') or ''}",
        f"Basic password: {auth.get('basicPassword') or ''}",
        f"Token env: {auth.get('tokenEnv') or ''}",
        f"Password env: {auth.get('passwordEnv') or ''}",
        f"Secret mode: {auth.get('secretMode') or ''}",
        f"Secret passphrase env: {auth.get('secretPassphraseEnv') or ''}",
    ]
    return lines


def build_profile_example_document(mode: str = "full") -> dict[str, Any]:
    """Build one annotated example profile document."""
    compact = _normalize_text(mode).lower() == "basic"
    profiles = {
        "prod": {
            "url": "https://grafana.example.com",
            "orgId": 1,
            "timeout": 30,
            "verifySsl": True,
            "insecure": False,
            "caCert": "/etc/ssl/certs/ca-certificates.crt",
            "auth": {
                "mode": "basic",
                "basicUser": "admin",
                "basicPassword": "******",
                "secretMode": "file",
            },
        }
    }
    if not compact:
        profiles["ci"] = {
            "url": "https://grafana.example.com",
            "orgId": 1,
            "timeout": 30,
            "verifySsl": True,
            "insecure": False,
            "caCert": "",
            "auth": {
                "mode": "token",
                "apiToken": "******",
                "tokenEnv": "GRAFANA_API_TOKEN",
                "secretMode": "os",
            },
        }
    return {
        "version": DEFAULT_PROFILE_VERSION,
        "defaultProfile": "prod",
        "profiles": profiles,
    }


def resolve_connection_details(
    args: Any,
    config_path: Optional[Path | str] = None,
) -> ProfileConnectionDetails:
    """Resolve URL, auth and transport options from explicit args plus profile."""
    from .auth_staging import resolve_auth_headers

    document = load_profile_document(config_path)
    profile_name = _normalize_text(getattr(args, "profile", None)) or None
    selected_profile = None
    if profile_name:
        _, selected_profile = select_profile(document, profile_name)

    auth = selected_profile.get("auth") if selected_profile else {}
    if not isinstance(auth, Mapping):
        auth = {}

    token = _normalize_text(getattr(args, "api_token", None)) or _normalize_text(
        auth.get("apiToken") or auth.get("apiTokenEncoded")
    )
    if not token and _normalize_text(auth.get("tokenEnv")):
        token = _normalize_text(
            __import__("os").environ.get(_normalize_text(auth.get("tokenEnv")))
        )

    username = _normalize_text(getattr(args, "username", None)) or _normalize_text(
        auth.get("basicUser")
    )
    password = _normalize_text(getattr(args, "password", None)) or _normalize_text(
        auth.get("basicPassword") or auth.get("basicPasswordEncoded")
    )
    if not password and _normalize_text(auth.get("passwordEnv")):
        password = _normalize_text(
            __import__("os").environ.get(_normalize_text(auth.get("passwordEnv")))
        )

    headers, _auth_mode = resolve_auth_headers(
        token=token or None,
        prompt_token=bool(getattr(args, "prompt_token", False))
        or bool(auth.get("promptToken", False)),
        username=username or None,
        password=password or None,
        prompt_password=bool(getattr(args, "prompt_password", False))
        or bool(auth.get("promptPassword", False)),
    )

    profile_url = _normalize_text(selected_profile.get("url") if selected_profile else "")
    profile_timeout = selected_profile.get("timeout") if selected_profile else None
    profile_verify_ssl = bool(
        selected_profile.get("verifySsl") if selected_profile else False
    )
    profile_insecure = bool(
        selected_profile.get("insecure") if selected_profile else False
    )
    profile_ca_cert = _normalize_text(
        selected_profile.get("caCert") if selected_profile else None
    )
    profile_org_id = _normalize_text(
        selected_profile.get("orgId") if selected_profile else None
    )

    return ProfileConnectionDetails(
        url=_normalize_text(getattr(args, "url", None)) or profile_url or "http://localhost:3000",
        headers=headers,
        timeout=int(getattr(args, "timeout", None) or profile_timeout or 30),
        verify_ssl=bool(
            getattr(args, "verify_ssl", None)
            if getattr(args, "verify_ssl", None) is not None
            else profile_verify_ssl
        ),
        insecure=bool(
            getattr(args, "insecure", None)
            if getattr(args, "insecure", None) is not None
            else profile_insecure
        ),
        ca_cert=_normalize_text(getattr(args, "ca_cert", None)) or profile_ca_cert or None,
        org_id=_normalize_text(getattr(args, "org_id", None)) or profile_org_id or None,
        profile_name=profile_name,
    )


__all__ = [
    "DEFAULT_CONFIG_FILENAME",
    "DEFAULT_PROFILE_SCOPE_NAME",
    "DEFAULT_PROFILE_VERSION",
    "DEFAULT_SECRETS_DIRNAME",
    "LATEST_RUN_POINTER_FILENAME",
    "PROFILE_CONFIG_ENV_VAR",
    "ProfileConnectionDetails",
    "build_profile_example_document",
    "build_profile_summary",
    "default_artifact_root_path_for_config_path",
    "list_profile_names",
    "load_profile_document",
    "profile_scope_name",
    "profile_scope_path",
    "read_latest_run_id",
    "resolve_artifact_root_path",
    "resolve_artifact_run_id",
    "resolve_config_path",
    "resolve_connection_details",
    "resolve_input_lane_path",
    "render_profile_summary_text",
    "save_profile_document",
    "select_profile",
]
