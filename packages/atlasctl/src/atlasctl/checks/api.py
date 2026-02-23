from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable


@dataclass(frozen=True)
class CheckResult:
    code: str
    messages: tuple[str, ...] = ()


@dataclass(frozen=True)
class CheckAuthoringMetadata:
    check_id: str
    domain: str
    intent: str
    remediation_short: str
    remediation_link: str
    effects: tuple[str, ...]
    result_code: str
    budget_ms: int
    category: str


def check(
    *,
    check_id: str,
    domain: str,
    intent: str,
    remediation_short: str = "Review check output and apply documented remediation.",
    remediation_link: str = "packages/atlasctl/docs/checks/check-id-migration-rules.md",
    effects: Iterable[str] = (),
    result_code: str = "CHECK_GENERIC",
    budget_ms: int = 1000,
    category: str = "check",
) -> Callable[[Callable[[Path], tuple[int, list[str]]]], Callable[[Path], tuple[int, list[str]]]]:
    def _decorate(fn: Callable[[Path], tuple[int, list[str]]]) -> Callable[[Path], tuple[int, list[str]]]:
        setattr(
            fn,
            "__atlasctl_check_meta__",
            CheckAuthoringMetadata(
                check_id=check_id,
                domain=domain,
                intent=intent.strip(),
                remediation_short=remediation_short.strip(),
                remediation_link=remediation_link.strip(),
                effects=tuple(str(item).strip() for item in effects if str(item).strip()),
                result_code=result_code.strip() or "CHECK_GENERIC",
                budget_ms=int(budget_ms),
                category=category.strip() or "check",
            ),
        )
        return fn

    return _decorate


__all__ = ["check", "CheckResult", "CheckAuthoringMetadata"]
