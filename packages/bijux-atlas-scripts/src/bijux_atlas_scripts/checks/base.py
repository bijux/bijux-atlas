from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable

CheckFunc = Callable[[Path], tuple[int, list[str]]]


@dataclass(frozen=True)
class CheckDef:
    check_id: str
    domain: str
    budget_ms: int
    fn: CheckFunc
