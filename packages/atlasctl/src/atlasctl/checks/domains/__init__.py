from __future__ import annotations

from ..model import CheckDef
from . import configs, docs, internal, make, ops, policies, python, repo, root


def register_all() -> tuple[CheckDef, ...]:
    checks = (
        *configs.CHECKS,
        *docs.CHECKS,
        *internal.CHECKS,
        *make.CHECKS,
        *ops.CHECKS,
        *policies.CHECKS,
        *python.CHECKS,
        *repo.CHECKS,
        *root.CHECKS,
    )
    by_id: dict[str, CheckDef] = {}
    for check in checks:
        by_id[str(check.check_id)] = check
    return tuple(sorted(by_id.values(), key=lambda check: str(check.check_id)))


CHECKS: tuple[CheckDef, ...] = register_all()

__all__ = ["CHECKS", "register_all"]
