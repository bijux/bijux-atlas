from __future__ import annotations

from ..repo.legacy_native import (
    check_make_command_allowlist,
    check_make_forbidden_paths,
    check_make_help,
    check_make_scripts_references,
)
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("make.scripts_refs", "make", "forbid scripts/ references in make recipes", 1000, check_make_scripts_references, fix_hint="Replace scripts/ invocations with atlasctl commands."),
    CheckDef("make.help_determinism", "make", "ensure deterministic make help output", 2000, check_make_help, fix_hint="Regenerate and normalize make help output."),
    CheckDef("make.forbidden_paths", "make", "forbid direct forbidden paths in make recipes", 1000, check_make_forbidden_paths, fix_hint="Route commands through allowed wrappers."),
    CheckDef("make.command_allowlist", "make", "enforce allowed direct recipe commands", 1500, check_make_command_allowlist, fix_hint="Use allowed command wrappers in make targets."),
)
