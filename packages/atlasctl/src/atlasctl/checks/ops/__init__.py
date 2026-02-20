from __future__ import annotations

from ..repo.legacy_native import (
    check_committed_generated_hygiene,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops.no_tracked_generated", "ops", "forbid tracked files in generated ops dirs", 800, check_ops_generated_tracked, fix_hint="Untrack generated ops files."),
    CheckDef("ops.no_tracked_timestamps", "ops", "forbid tracked timestamped paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked paths."),
    CheckDef("ops.committed_generated_hygiene", "ops", "validate deterministic committed generated assets", 1000, check_committed_generated_hygiene, fix_hint="Regenerate committed outputs deterministically."),
)
