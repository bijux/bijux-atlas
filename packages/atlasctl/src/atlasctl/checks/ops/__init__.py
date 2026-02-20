from __future__ import annotations

from ...check.native import (
    check_committed_generated_hygiene,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops/no-tracked-generated", "ops", 800, check_ops_generated_tracked),
    CheckDef("ops/no-tracked-timestamps", "ops", 1000, check_tracked_timestamp_paths),
    CheckDef("ops/committed-generated-hygiene", "ops", 1000, check_committed_generated_hygiene),
)
