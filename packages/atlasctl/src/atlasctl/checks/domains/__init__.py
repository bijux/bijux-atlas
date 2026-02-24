from __future__ import annotations

from ..model import CheckDef
from . import configs, docs, internal, make, ops, policies, python, repo, root


def register_all() -> tuple[CheckDef, ...]:
    checks = (
        *configs.register(),
        *docs.register(),
        *internal.register(),
        *make.register(),
        *ops.register(),
        *policies.register(),
        *python.register(),
        *repo.register(),
        *root.register(),
    )
    by_id: dict[str, CheckDef] = {}
    for check in checks:
        by_id[str(check.check_id)] = check
    return tuple(sorted(by_id.values(), key=lambda check: str(check.check_id)))


CHECKS: tuple[CheckDef, ...] = register_all()

__all__ = ["CHECKS", "register_all"]
