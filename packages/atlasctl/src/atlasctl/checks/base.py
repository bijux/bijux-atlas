from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Callable

CheckFunc = Callable[[Path], tuple[int, list[str]]]


class Severity(str, Enum):
    ERROR = "error"
    WARN = "warn"
    INFO = "info"


@dataclass(frozen=True)
class CheckDef:
    check_id: str
    domain: str
    budget_ms: int
    fn: CheckFunc
    severity: Severity = Severity.ERROR
    evidence: tuple[str, ...] = ()


@dataclass(frozen=True)
class CheckResult:
    check_id: str
    domain: str
    status: str
    duration_ms: int
    budget_ms: int
    budget_status: str
    errors: list[str]
    severity: str
    evidence: tuple[str, ...] = ()
