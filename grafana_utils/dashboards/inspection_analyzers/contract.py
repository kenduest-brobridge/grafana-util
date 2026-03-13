import re
from typing import Any, Dict, List, Set
DATASOURCE_FAMILY_PROMETHEUS = "prometheus"
DATASOURCE_FAMILY_LOKI = "loki"
DATASOURCE_FAMILY_FLUX = "flux"
DATASOURCE_FAMILY_SQL = "sql"
DATASOURCE_FAMILY_UNKNOWN = "unknown"
QUERY_ANALYSIS_FIELDS = ("metrics", "measurements", "buckets")
def extract_string_values(query: str, pattern: str) -> List[str]:
    if not query:
        return []
    values = []
    for match in re.findall(pattern, query):
        if isinstance(match, tuple):
            for item in match:
                if item:
                    values.append(str(item))
                    break
        elif match:
            values.append(str(match))
    return values
def unique_strings(values: List[str]) -> List[str]:
    seen = set()  # type: Set[str]
    ordered = []
    for value in values:
        text = str(value or "").strip()
        if not text or text in seen:
            continue
        seen.add(text)
        ordered.append(text)
    return ordered
def normalize_query_analysis(result: Dict[str, Any]) -> Dict[str, List[str]]:
    normalized = {}
    for field in QUERY_ANALYSIS_FIELDS:
        value = (result or {}).get(field)
        if isinstance(value, list):
            normalized[field] = unique_strings([str(item) for item in value])
        elif value is None:
            normalized[field] = []
        else:
            normalized[field] = unique_strings([str(value)])
    return normalized
def build_query_field_and_text(target: Dict[str, Any]) -> List[str]:
    for field in ("expr", "expression", "query", "rawSql", "sql", "rawQuery", "jql", "logql", "search", "definition", "command"):
        value = target.get(field)
        if value is None:
            continue
        text = str(value).strip()
        if text:
            return [field, text]
    return ["", ""]
def extract_metric_names(query: str) -> List[str]:
    return unique_strings(extract_string_values(query, r'__name__\s*=\s*"([A-Za-z_:][A-Za-z0-9_:]*)"'))
def extract_prometheus_metric_names(query: str) -> List[str]:
    return unique_strings(extract_string_values(query, r'([A-Za-z_:][A-Za-z0-9_:]*)\{'))
def extract_measurements(query: str) -> List[str]:
    return unique_strings(extract_string_values(query, r'_measurement\s*==\s*"([^"]+)"'))
def extract_buckets(query: str) -> List[str]:
    return unique_strings(extract_string_values(query, r'from\s*\(\s*bucket\s*:\s*"([^"]+)"'))
def build_default_query_analysis(target: Dict[str, Any], query_text: str) -> Dict[str, List[str]]:
    del target
    return normalize_query_analysis({"metrics": extract_metric_names(query_text), "measurements": extract_measurements(query_text), "buckets": extract_buckets(query_text)})
