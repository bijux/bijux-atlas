"""Pagination helpers for atlas-client."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(slots=True)
class Page:
    """A page from an Atlas response."""

    items: list[dict[str, Any]]
    next_token: str | None


def next_page_token(payload: dict[str, Any]) -> str | None:
    token = payload.get("next_page_token")
    return token if isinstance(token, str) else None


def page_items(payload: dict[str, Any]) -> list[dict[str, Any]]:
    raw = payload.get("items", [])
    if isinstance(raw, list):
        return [entry for entry in raw if isinstance(entry, dict)]
    return []
