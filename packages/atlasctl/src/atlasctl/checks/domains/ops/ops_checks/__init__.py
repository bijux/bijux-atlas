from __future__ import annotations

from pathlib import Path

from .check_ops_manifests_schema import run as run_ops_manifests_schema_check
from ....repo.native import (
    check_committed_generated_hygiene,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ....core.base import CheckDef


def check_ops_manifests_schema(repo_root: Path) -> tuple[int, list[str]]:
    del repo_root
    code = run_ops_manifests_schema_check()
    return code, []


CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops.no_tracked_generated", "ops", "forbid tracked files in generated ops dirs", 800, check_ops_generated_tracked, fix_hint="Untrack generated ops files."),
    CheckDef("ops.no_tracked_timestamps", "ops", "forbid tracked timestamped paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked paths."),
    CheckDef("ops.committed_generated_hygiene", "ops", "validate deterministic committed generated assets", 1000, check_committed_generated_hygiene, fix_hint="Regenerate committed outputs deterministically."),
    CheckDef("ops.manifests_schema", "ops", "validate ops manifests against atlas.ops.manifest.v1 schema", 1000, check_ops_manifests_schema, fix_hint="Fix ops/manifests/*.json|*.yaml to satisfy atlas.ops.manifest.v1."),
)
