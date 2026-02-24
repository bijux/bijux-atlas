from __future__ import annotations

from ..model import CheckDef
from ..tools.repo_domain import CHECKS as REPO_CHECKS

_ROOT_OWNED_CHECK_IDS = {
    "repo.forbidden_top_dirs",
    "repo.forbidden_root_files",
    "repo.forbidden_root_names",
    "repo.no_forbidden_paths",
    "repo.root_determinism",
    "repo.root_shape",
    "repo.atlasctl_package_root_shape",
}

CHECKS: tuple[CheckDef, ...] = tuple(check for check in REPO_CHECKS if str(check.check_id) not in _ROOT_OWNED_CHECK_IDS)

__all__ = ["CHECKS"]
