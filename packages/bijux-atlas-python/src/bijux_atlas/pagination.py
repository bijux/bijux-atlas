"""Pagination helpers."""

from dataclasses import dataclass


@dataclass(frozen=True)
class PageCursor:
    """Cursor metadata for paged responses."""

    next_token: str | None = None
