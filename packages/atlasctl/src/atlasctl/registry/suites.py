from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SuiteRecord:
    name: str
    includes: tuple[str, ...]
    item_count: int
    complete: bool
    tags: tuple[str, ...]
    internal: bool = False
