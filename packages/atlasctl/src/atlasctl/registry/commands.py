from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CommandRecord:
    name: str
    help: str
    tags: tuple[str, ...]
    owner: str
    aliases: tuple[str, ...]
    stable: bool = True
    internal: bool = False
