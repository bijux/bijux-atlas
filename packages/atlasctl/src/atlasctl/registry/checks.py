from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CheckRecord:
    id: str
    title: str
    domain: str
    tags: tuple[str, ...]
    effects: tuple[str, ...]
    owners: tuple[str, ...]
    internal: bool = False
