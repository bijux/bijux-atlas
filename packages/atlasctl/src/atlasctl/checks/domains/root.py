from __future__ import annotations

from ..model import CheckDef
from ..tools.repo import (
    FORBIDDEN_PATHS_DESCRIPTION,
    FORBIDDEN_ROOT_FILES_DESCRIPTION,
    FORBIDDEN_ROOT_NAMES_DESCRIPTION,
    ROOT_DETERMINISM_DESCRIPTION,
    ROOT_SHAPE_DESCRIPTION,
    check_atlasctl_package_root_shape,
    check_forbidden_top_dirs,
    run_forbidden_paths,
    run_forbidden_root_files,
    run_forbidden_root_names,
    run_root_determinism,
    run_root_shape,
)

CHECKS: tuple[CheckDef, ...] = (
    CheckDef(
        "repo.forbidden_top_dirs",
        "repo",
        "forbid top-level forbidden directories",
        500,
        check_forbidden_top_dirs,
        fix_hint="Remove forbidden root directories.",
    ),
    CheckDef(
        "repo.forbidden_root_files",
        "repo",
        FORBIDDEN_ROOT_FILES_DESCRIPTION,
        500,
        run_forbidden_root_files,
        fix_hint="Remove junk files from repository root.",
    ),
    CheckDef(
        "repo.forbidden_root_names",
        "repo",
        FORBIDDEN_ROOT_NAMES_DESCRIPTION,
        500,
        run_forbidden_root_names,
        fix_hint="Move legacy top-level entries into canonical homes.",
    ),
    CheckDef(
        "repo.no_forbidden_paths",
        "repo",
        FORBIDDEN_PATHS_DESCRIPTION,
        500,
        run_forbidden_paths,
        fix_hint="Replace legacy path references with canonical paths.",
        effects=("subprocess",),
    ),
    CheckDef(
        "repo.root_determinism",
        "repo",
        ROOT_DETERMINISM_DESCRIPTION,
        700,
        run_root_determinism,
        fix_hint="Stabilize generated root docs outputs and remove nondeterminism.",
        effects=("subprocess",),
    ),
    CheckDef(
        "repo.root_shape",
        "repo",
        ROOT_SHAPE_DESCRIPTION,
        500,
        run_root_shape,
        fix_hint="Update repository root entries to match root policy contract.",
        effects=("subprocess",),
    ),
    CheckDef(
        "repo.atlasctl_package_root_shape",
        "repo",
        "enforce atlasctl package root shape",
        500,
        check_atlasctl_package_root_shape,
        fix_hint="Keep packages/atlasctl/src/atlasctl root entries aligned with package shape policy.",
    ),
)


def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
