from __future__ import annotations

from ..repo.legacy_native import (
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_help,
    check_make_scripts_references,
)
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("make/scripts-refs", "make", 1000, check_make_scripts_references),
    CheckDef("make/help-determinism", "make", 2000, check_make_help),
    CheckDef("make/forbidden-paths", "make", 1000, check_make_forbidden_paths),
    CheckDef("make/command-allowlist", "make", 1500, check_make_command_allowlist),
)
