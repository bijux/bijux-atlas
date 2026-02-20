from __future__ import annotations

from dataclasses import dataclass


ERR_INTERNAL = 99
ERR_USAGE = 2
ERR_POLICY = 16


@dataclass
class ScriptError(Exception):
    message: str
    code: int = ERR_INTERNAL

    def __str__(self) -> str:
        return self.message
