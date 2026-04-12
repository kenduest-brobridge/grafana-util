from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path


UI_TEXT_PATH = Path(__file__).resolve().parent / "contracts" / "doc-ui.json"


@lru_cache(maxsize=1)
def load_ui_text() -> dict:
    raw = json.loads(UI_TEXT_PATH.read_text(encoding="utf-8"))
    if raw.get("schema") != 1:
        raise ValueError(f"Unsupported doc UI schema in {UI_TEXT_PATH}")
    locales = raw.get("locales")
    if not isinstance(locales, dict):
        raise TypeError(f"{UI_TEXT_PATH} must define a locale map")
    return raw


def ui_text(locale: str, key: str):
    locales = load_ui_text()["locales"]
    selected = locales.get(locale) or locales.get("en")
    if selected is None:
        raise KeyError(f"No UI text found for locale {locale}")
    if key not in selected:
        raise KeyError(f"Missing UI text key {key} for locale {locale}")
    return selected[key]
