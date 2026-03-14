from typing import Any, Dict

from .contract import build_default_query_analysis


def analyze_query(
    panel: Dict[str, Any],
    target: Dict[str, Any],
    query_field: str,
    query_text: str,
) -> Dict[str, Any]:
    del panel, query_field
    return build_default_query_analysis(target, query_text)
