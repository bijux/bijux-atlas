from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Callable, Protocol

CheckFunc = Callable[[Path], tuple[int, list[str]]]


class Severity(str, Enum):
    ERROR = "error"
    WARN = "warn"
    INFO = "info"


class CheckCategory(str, Enum):
    POLICY = "policy"
    HYGIENE = "hygiene"
    CONTRACT = "contract"
    DRIFT = "drift"
    SECURITY = "security"


@dataclass(frozen=True)
class CheckDef:
    check_id: str
    domain: str
    description: str
    budget_ms: int
    fn: CheckFunc
    severity: Severity = Severity.ERROR
    category: CheckCategory = CheckCategory.HYGIENE
    fix_hint: str = "Review check output and apply the documented fix."
    slow: bool = False
    tags: tuple[str, ...] = ()
    external_tools: tuple[str, ...] = ()
    evidence: tuple[str, ...] = ()


@dataclass(frozen=True)
class CheckResult:
    id: str
    domain: str
    status: str
    errors: list[str]
    warnings: list[str]
    evidence_paths: list[str]
    metrics: dict[str, object]
    description: str
    fix_hint: str
    category: str
    severity: str


class Check(Protocol):
    id: str
    domain: str
    description: str

    def run(self, repo_root: Path) -> CheckResult:
        ...
    evidence: tuple[str, ...] = ()
