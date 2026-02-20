from __future__ import annotations

from collections import defaultdict
from pathlib import Path

from .base import CheckDef
from .checks import CHECKS as CHECKS_CHECKS
from .configs import CHECKS as CHECKS_CONFIGS
from .docker import CHECKS as CHECKS_DOCKER
from .docs import CHECKS as CHECKS_DOCS
from .make import CHECKS as CHECKS_MAKE
from .ops import CHECKS as CHECKS_OPS
from .repo import CHECKS as CHECKS_REPO


_CHECKS: tuple[CheckDef, ...] = (
    *CHECKS_REPO,
    *CHECKS_MAKE,
    *CHECKS_DOCS,
    *CHECKS_OPS,
    *CHECKS_CHECKS,
    *CHECKS_CONFIGS,
    *CHECKS_DOCKER,
)


def list_checks() -> tuple[CheckDef, ...]:
    return _CHECKS


def list_domains() -> list[str]:
    return sorted({"all", *{c.domain for c in _CHECKS}})


def checks_by_domain() -> dict[str, list[CheckDef]]:
    grouped: dict[str, list[CheckDef]] = defaultdict(list)
    for check in _CHECKS:
        grouped[check.domain].append(check)
    return dict(grouped)


def run_checks_for_domain(repo_root: Path, domain: str) -> list[CheckDef]:
    if domain == "all":
        return list(_CHECKS)
    return [c for c in _CHECKS if c.domain == domain]
