"""Supported datasource catalog used by datasource CLI surfaces."""

from copy import deepcopy

SUPPORTED_DATASOURCE_PRESET_PROFILES = ("starter", "full")


TRACING_EXPLORATION_DEFAULTS = {
    "nodeGraph": {"enabled": True},
    "traceQuery": {"timeShiftEnabled": True},
}

TEMPO_TRACING_DEFAULTS = {
    **TRACING_EXPLORATION_DEFAULTS,
    "search": {"hide": False},
    "traceQuery": {
        "timeShiftEnabled": True,
        "spanStartTimeShift": "-1h",
        "spanEndTimeShift": "1h",
    },
    "streamingEnabled": {"search": True},
}

SQL_POOL_DEFAULTS = {
    "maxOpenConns": 100,
    "maxIdleConns": 100,
    "maxIdleConnsAuto": True,
    "connMaxLifetime": 14400,
}

SQL_STARTER_DEFAULTS = {
    **SQL_POOL_DEFAULTS,
    "database": "grafana",
}

LOKI_FULL_DEFAULTS = {
    "access": "proxy",
    "jsonData": {
        "maxLines": 1000,
        "timeout": 60,
        "derivedFields": [
            {
                "name": "TraceID",
                "matcherRegex": "traceID=(\\w+)",
                "datasourceUid": "tempo",
                "url": "$${__value.raw}",
                "urlDisplayLabel": "View Trace",
            }
        ],
    },
}

TEMPO_FULL_DEFAULTS = {
    "access": "proxy",
    "jsonData": {
        **TEMPO_TRACING_DEFAULTS,
        "serviceMap": {"datasourceUid": "prometheus"},
        "tracesToLogsV2": {
            "datasourceUid": "loki",
            "spanStartTimeShift": "-1h",
            "spanEndTimeShift": "1h",
        },
        "tracesToMetrics": {
            "datasourceUid": "prometheus",
            "spanStartTimeShift": "-1h",
            "spanEndTimeShift": "1h",
        },
    },
}

MYSQL_FULL_DEFAULTS = {
    "access": "proxy",
    "jsonData": {
        **SQL_STARTER_DEFAULTS,
        "tlsAuth": True,
        "tlsSkipVerify": True,
    },
}


def _catalog_entry(
    category,
    type_id,
    display_name,
    *,
    aliases=(),
    profile,
    query_language,
    suggested_flags=(),
    requires_datasource_url=True,
    add_defaults=None,
    full_add_defaults=None,
):
    """Build one normalized datasource catalog entry."""
    starter_defaults = deepcopy(add_defaults or {})
    full_defaults = deepcopy(
        full_add_defaults if full_add_defaults is not None else starter_defaults
    )
    return {
        "category": category,
        "type": type_id,
        "display_name": display_name,
        "aliases": tuple(aliases),
        "profile": profile,
        "query_language": query_language,
        "suggested_flags": tuple(suggested_flags),
        "requires_datasource_url": bool(requires_datasource_url),
        "add_defaults": starter_defaults,
        "full_add_defaults": full_defaults,
    }


SUPPORTED_DATASOURCE_CATALOG = (
    _catalog_entry(
        "Metrics",
        "prometheus",
        "Prometheus",
        aliases=("grafana-prometheus-datasource",),
        profile="metrics-http",
        query_language="promql",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": {"httpMethod": "POST"},
        },
    ),
    _catalog_entry(
        "Metrics",
        "influxdb",
        "InfluxDB",
        aliases=("grafana-influxdb-datasource", "flux"),
        profile="metrics-http",
        query_language="flux-or-influxql",
        suggested_flags=(
            "--user",
            "--password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": {
                "version": "Flux",
                "organization": "main-org",
                "defaultBucket": "metrics",
            },
        },
    ),
    _catalog_entry(
        "Metrics",
        "graphite",
        "Graphite",
        profile="metrics-http",
        query_language="graphite",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": {"graphiteVersion": "1.1"},
        },
    ),
    _catalog_entry(
        "Metrics",
        "opentsdb",
        "OpenTSDB",
        profile="metrics-http",
        query_language="opentsdb",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": {"tsdbVersion": 2},
        },
    ),
    _catalog_entry(
        "Logs",
        "loki",
        "Loki",
        aliases=("grafana-loki-datasource",),
        profile="logs-http",
        query_language="logql",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": {
                "maxLines": 1000,
                "timeout": 60,
            },
        },
        full_add_defaults=LOKI_FULL_DEFAULTS,
    ),
    _catalog_entry(
        "Logs",
        "elasticsearch",
        "Elasticsearch",
        profile="logs-search-api",
        query_language="lucene-or-query-dsl",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--user",
            "--password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={"access": "proxy", "jsonData": {"timeField": "@timestamp"}},
    ),
    _catalog_entry(
        "Logs",
        "opensearch",
        "OpenSearch",
        profile="logs-search-api",
        query_language="ppl-or-query-dsl",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--user",
            "--password",
            "--with-credentials",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={"access": "proxy", "jsonData": {"timeField": "@timestamp"}},
    ),
    _catalog_entry(
        "Tracing",
        "jaeger",
        "Jaeger",
        profile="tracing-http",
        query_language="trace-search",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": TRACING_EXPLORATION_DEFAULTS,
        },
    ),
    _catalog_entry(
        "Tracing",
        "zipkin",
        "Zipkin",
        profile="tracing-http",
        query_language="trace-search",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": TRACING_EXPLORATION_DEFAULTS,
        },
    ),
    _catalog_entry(
        "Tracing",
        "tempo",
        "Tempo",
        profile="tracing-http",
        query_language="traceql",
        suggested_flags=(
            "--basic-auth",
            "--basic-auth-user",
            "--basic-auth-password",
            "--http-header",
            "--tls-skip-verify",
            "--server-name",
        ),
        add_defaults={
            "access": "proxy",
            "jsonData": TEMPO_TRACING_DEFAULTS,
        },
        full_add_defaults=TEMPO_FULL_DEFAULTS,
    ),
    _catalog_entry(
        "Databases",
        "mysql",
        "MySQL",
        aliases=("grafana-mysql-datasource",),
        profile="sql-database",
        query_language="sql",
        suggested_flags=("--user", "--password", "--tls-skip-verify", "--server-name"),
        add_defaults={"access": "proxy", "jsonData": SQL_STARTER_DEFAULTS},
        full_add_defaults=MYSQL_FULL_DEFAULTS,
    ),
    _catalog_entry(
        "Databases",
        "postgresql",
        "PostgreSQL",
        aliases=("postgres", "grafana-postgresql-datasource"),
        profile="sql-database",
        query_language="sql",
        suggested_flags=("--user", "--password", "--tls-skip-verify", "--server-name"),
        add_defaults={
            "access": "proxy",
            "jsonData": {
                **SQL_STARTER_DEFAULTS,
                "sslmode": "disable",
            },
        },
        full_add_defaults={
            "access": "proxy",
            "jsonData": {
                **SQL_STARTER_DEFAULTS,
                "sslmode": "disable",
                "postgresVersion": 903,
                "timescaledb": False,
            },
        },
    ),
    _catalog_entry(
        "Databases",
        "mssql",
        "MSSQL",
        profile="sql-database",
        query_language="sql",
        suggested_flags=("--user", "--password", "--tls-skip-verify", "--server-name"),
        add_defaults={
            "access": "proxy",
            "jsonData": {
                **SQL_POOL_DEFAULTS,
                "database": "grafana",
                "connectionTimeout": 0,
            },
        },
    ),
    _catalog_entry(
        "Databases",
        "sqlite",
        "SQLite",
        profile="sql-database",
        query_language="sql",
        suggested_flags=("--user", "--password"),
        requires_datasource_url=False,
        add_defaults={
            "access": "proxy",
            "jsonData": {
                "path": "/var/lib/sqlite/grafana.db",
            },
        },
    ),
)


def find_supported_datasource_entry(type_or_alias):
    """Return the matching catalog entry for a datasource type or alias."""
    candidate = str(type_or_alias or "").strip().lower()
    if not candidate:
        return None
    for entry in SUPPORTED_DATASOURCE_CATALOG:
        if candidate == entry["type"]:
            return entry
        if candidate in entry["aliases"]:
            return entry
    return None


def normalize_supported_datasource_type(type_or_alias):
    """Normalize a supported datasource alias into its canonical type id."""
    entry = find_supported_datasource_entry(type_or_alias)
    if entry is None:
        return str(type_or_alias or "").strip()
    return entry["type"]


def normalize_supported_datasource_preset_profile(preset_profile):
    """Normalize a supported datasource preset profile name."""
    candidate = str(preset_profile or "").strip().lower()
    if not candidate:
        return "starter"
    if candidate in SUPPORTED_DATASOURCE_PRESET_PROFILES:
        return candidate
    return candidate


def build_add_defaults_for_supported_type(type_or_alias, preset_profile="starter"):
    """Return a deep-copied add-defaults scaffold for a supported datasource type."""
    entry = find_supported_datasource_entry(type_or_alias)
    if entry is None:
        return {}
    profile = normalize_supported_datasource_preset_profile(preset_profile)
    if profile == "full":
        return deepcopy(entry.get("full_add_defaults") or entry.get("add_defaults") or {})
    return deepcopy(entry.get("add_defaults") or {})


def supported_preset_profiles_for_type(type_or_alias):
    """Return supported preset profiles for a datasource type."""
    entry = find_supported_datasource_entry(type_or_alias)
    if entry is None:
        return ()
    starter = deepcopy(entry.get("add_defaults") or {})
    full = deepcopy(entry.get("full_add_defaults") or starter)
    if full != starter:
        return SUPPORTED_DATASOURCE_PRESET_PROFILES
    return ("starter",)


def _iter_json_data_defaults(prefix, value):
    """Yield flattened jsonData defaults for readable text rendering."""
    if isinstance(value, dict):
        for child_key, child_value in value.items():
            child_prefix = f"{prefix}.{child_key}" if prefix else str(child_key)
            yield from _iter_json_data_defaults(child_prefix, child_value)
        return
    yield f"jsonData.{prefix}={value}"


def render_supported_datasource_catalog_text():
    """Render the supported datasource catalog as grouped plain text."""
    lines = ["Grafana Data Sources Summary", ""]
    current_category = None
    for entry in SUPPORTED_DATASOURCE_CATALOG:
        if entry["category"] != current_category:
            if current_category is not None:
                lines.append("")
            current_category = entry["category"]
            lines.append("%s:" % current_category)
        line = "  - %s (%s)" % (entry["display_name"], entry["type"])
        line += " profile=%s query=%s" % (
            entry["profile"],
            entry["query_language"],
        )
        if entry.get("requires_datasource_url", True):
            line += " url=required"
        defaults = entry.get("add_defaults") or {}
        default_bits = []
        if defaults.get("access"):
            default_bits.append("access=%s" % defaults["access"])
        json_data_defaults = defaults.get("jsonData") or {}
        for key, value in json_data_defaults.items():
            default_bits.extend(_iter_json_data_defaults(str(key), value))
        if default_bits:
            line += " defaults: %s" % ", ".join(default_bits)
        preset_profiles = supported_preset_profiles_for_type(entry["type"])
        if preset_profiles:
            line += " preset-profiles: %s" % ", ".join(preset_profiles)
        aliases = entry["aliases"]
        if aliases:
            line += " aliases: %s" % ", ".join(aliases)
        suggested_flags = entry.get("suggested_flags") or ()
        if suggested_flags:
            line += " flags: %s" % ", ".join(suggested_flags)
        lines.append(line)
    return lines


def build_supported_datasource_catalog_document():
    """Build the supported datasource catalog as structured JSON-like data."""
    categories = []
    current = None
    for entry in SUPPORTED_DATASOURCE_CATALOG:
        if current is None or current["category"] != entry["category"]:
            current = {"category": entry["category"], "types": []}
            categories.append(current)
        current["types"].append(
            {
                "type": entry["type"],
                "displayName": entry["display_name"],
                "aliases": list(entry["aliases"]),
                "profile": entry["profile"],
                "queryLanguage": entry["query_language"],
                "requiresDatasourceUrl": bool(
                    entry.get("requires_datasource_url", True)
                ),
                "suggestedFlags": list(entry.get("suggested_flags") or ()),
                "presetProfiles": list(
                    supported_preset_profiles_for_type(entry["type"])
                ),
                "addDefaults": deepcopy(entry.get("add_defaults") or {}),
                "fullAddDefaults": deepcopy(entry.get("full_add_defaults") or {}),
            }
        )
    return {
        "kind": "grafana-utils-datasource-supported-types",
        "categories": categories,
    }
