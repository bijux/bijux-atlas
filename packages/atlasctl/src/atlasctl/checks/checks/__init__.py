from __future__ import annotations

from ..repo.legacy_native import check_script_help, check_script_ownership
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("repo.script_help_coverage", "repo", "validate script help contract coverage", 1500, check_script_help, fix_hint="Add --help contract output to required scripts."),
    CheckDef("repo.script_ownership_coverage", "repo", "validate script ownership coverage", 1500, check_script_ownership, fix_hint="Update ownership metadata for uncovered scripts."),
)
