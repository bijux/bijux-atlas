from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ScriptError(Exception):
    message: str
    code: int
    kind: str = "generic_error"

    def __str__(self) -> str:
        return self.message
