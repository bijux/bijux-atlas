from __future__ import annotations

from collections import defaultdict
from pathlib import Path

from .base import CheckDef
from .contracts import CHECKS as CHECKS_CONTRACTS
from .configs import CHECKS as CHECKS_CONFIGS
from .docker import CHECKS as CHECKS_DOCKER
from .docs import CHECKS as CHECKS_DOCS
from .licensing import CHECKS as CHECKS_LICENSE
from .make import CHECKS as CHECKS_MAKE
from .ops import CHECKS as CHECKS_OPS
from .python import CHECKS as CHECKS_PYTHON
from .repo import CHECKS as CHECKS_REPO


_CHECKS: tuple[CheckDef, ...] = (
    *CHECKS_REPO,
    *CHECKS_LICENSE,
    *CHECKS_MAKE,
    *CHECKS_DOCS,
    *CHECKS_OPS,
    *CHECKS_CONFIGS,
    *CHECKS_PYTHON,
    *CHECKS_DOCKER,
    *CHECKS_CONTRACTS,
)


def list_checks() -> tuple[CheckDef, ...]:
    seen: set[str] = set()
    duplicates: set[str] = set()
    for check in _CHECKS:
        if check.check_id in seen:
            duplicates.add(check.check_id)
        seen.add(check.check_id)
    if duplicates:
        dup_list = ", ".join(sorted(duplicates))
        raise ValueError(f"duplicate check ids in registry: {dup_list}")
    return tuple(sorted(_CHECKS, key=lambda c: c.check_id))


def list_domains() -> list[str]:
    return sorted({"all", *{c.domain for c in _CHECKS}})


def checks_by_domain() -> dict[str, list[CheckDef]]:
    grouped: dict[str, list[CheckDef]] = defaultdict(list)
    for check in _CHECKS:
        grouped[check.domain].append(check)
    return dict(grouped)


def run_checks_for_domain(repo_root: Path, domain: str) -> list[CheckDef]:
    if domain == "all":
        return list(list_checks())
    return [c for c in list_checks() if c.domain == domain]


def get_check(check_id: str) -> CheckDef | None:
    for check in list_checks():
        if check.check_id == check_id:
            return check
    return None
