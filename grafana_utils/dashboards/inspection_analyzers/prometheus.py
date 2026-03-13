from typing import Any, Dict
from .contract import extract_buckets, extract_measurements, extract_prometheus_metric_names, normalize_query_analysis
def analyze_query(panel: Dict[str, Any], target: Dict[str, Any], query_field: str, query_text: str) -> Dict[str, Any]:
    del panel, target, query_field
    return normalize_query_analysis({"metrics": extract_prometheus_metric_names(query_text), "measurements": extract_measurements(query_text), "buckets": extract_buckets(query_text)})
