from __future__ import annotations

import json
from dataclasses import dataclass

from docgen_common import REPO_ROOT


HANDBOOK_RENDER_PATH = REPO_ROOT / "scripts" / "contracts" / "handbook-render.json"


@dataclass(frozen=True)
class HandbookRenderPolicy:
    show_command_map: bool
    show_section_index: bool
    section_index_levels: tuple[int, ...]
    section_index_variant: str
    section_index_min_entries: int


def _expect_dict(value: object, field: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise TypeError(f"{field} must be an object")
    return value


def _expect_bool(value: object, field: str) -> bool:
    if not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean")
    return value


def _expect_int(value: object, field: str) -> int:
    if not isinstance(value, int):
        raise TypeError(f"{field} must be an integer")
    return value


def _expect_str(value: object, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def _expect_int_list(value: object, field: str) -> tuple[int, ...]:
    if not isinstance(value, list) or not value:
        raise TypeError(f"{field} must be a non-empty list")
    items: list[int] = []
    for index, item in enumerate(value):
        if not isinstance(item, int):
            raise TypeError(f"{field}[{index}] must be an integer")
        items.append(item)
    return tuple(items)


def _policy_from_dict(raw: dict[str, object], field: str) -> HandbookRenderPolicy:
    variant = _expect_str(raw.get("section_index_variant"), f"{field}.section_index_variant")
    if variant not in {"list", "grid"}:
        raise ValueError(f"{field}.section_index_variant must be 'list' or 'grid'")
    min_entries = _expect_int(raw.get("section_index_min_entries"), f"{field}.section_index_min_entries")
    if min_entries < 1:
        raise ValueError(f"{field}.section_index_min_entries must be >= 1")
    return HandbookRenderPolicy(
        show_command_map=_expect_bool(raw.get("show_command_map"), f"{field}.show_command_map"),
        show_section_index=_expect_bool(raw.get("show_section_index"), f"{field}.show_section_index"),
        section_index_levels=_expect_int_list(raw.get("section_index_levels"), f"{field}.section_index_levels"),
        section_index_variant=variant,
        section_index_min_entries=min_entries,
    )


def _load_handbook_render_contract() -> tuple[HandbookRenderPolicy, dict[str, HandbookRenderPolicy], dict[str, dict[str, str]]]:
    raw = json.loads(HANDBOOK_RENDER_PATH.read_text(encoding="utf-8"))
    if raw.get("schema") != 1:
        raise ValueError(f"Unsupported handbook render schema in {HANDBOOK_RENDER_PATH}")

    default_policy = _policy_from_dict(_expect_dict(raw.get("defaults"), "defaults"), "defaults")

    page_overrides_raw = _expect_dict(raw.get("page_overrides"), "page_overrides")
    page_overrides: dict[str, HandbookRenderPolicy] = {}
    for stem, override in page_overrides_raw.items():
        override_dict = _expect_dict(override, f"page_overrides.{stem}")
        merged = {
            "show_command_map": override_dict.get("show_command_map", default_policy.show_command_map),
            "show_section_index": override_dict.get("show_section_index", default_policy.show_section_index),
            "section_index_levels": override_dict.get("section_index_levels", list(default_policy.section_index_levels)),
            "section_index_variant": override_dict.get("section_index_variant", default_policy.section_index_variant),
            "section_index_min_entries": override_dict.get("section_index_min_entries", default_policy.section_index_min_entries),
        }
        page_overrides[stem] = _policy_from_dict(merged, f"page_overrides.{stem}")

    locale_strings_raw = _expect_dict(raw.get("locale_strings"), "locale_strings")
    locale_strings: dict[str, dict[str, str]] = {}
    for locale, values in locale_strings_raw.items():
        value_dict = _expect_dict(values, f"locale_strings.{locale}")
        locale_strings[locale] = {
            "previous_chapter": _expect_str(value_dict.get("previous_chapter"), f"locale_strings.{locale}.previous_chapter"),
            "next_chapter": _expect_str(value_dict.get("next_chapter"), f"locale_strings.{locale}.next_chapter"),
            "section_index_title": _expect_str(value_dict.get("section_index_title"), f"locale_strings.{locale}.section_index_title"),
            "section_index_summary": _expect_str(value_dict.get("section_index_summary"), f"locale_strings.{locale}.section_index_summary"),
        }

    return default_policy, page_overrides, locale_strings


HANDBOOK_RENDER_DEFAULTS, HANDBOOK_RENDER_PAGE_OVERRIDES, HANDBOOK_RENDER_LOCALE_STRINGS = _load_handbook_render_contract()


def handbook_render_policy(stem: str) -> HandbookRenderPolicy:
    return HANDBOOK_RENDER_PAGE_OVERRIDES.get(stem, HANDBOOK_RENDER_DEFAULTS)

