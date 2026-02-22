from __future__ import annotations

import re
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


def check_ops_no_direct_script_entrypoints(repo_root: Path) -> tuple[int, list[str]]:
    command_patterns = (
        re.compile(r"(?:^|\s)(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
        re.compile(r"\b(?:bash|sh)\s+(?:\./)?ops/(?!run/)[A-Za-z0-9_./-]+\.(?:sh|py)\b"),
    )
    errors: list[str] = []
    scan_roots = [
        repo_root / "docs" / "development",
        repo_root / "docs" / "control-plane",
        repo_root / ".github" / "workflows",
        repo_root / "makefiles",
    ]
    for base in scan_roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in {".md", ".mk", ".yml", ".yaml"}:
                continue
            rel = path.relative_to(repo_root).as_posix()
            if rel.startswith("docs/_generated/") or rel.startswith("docs/_lint/"):
                continue
            for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
                stripped = line.strip()
                if not stripped or stripped.startswith("#"):
                    continue
                for pattern in command_patterns:
                    match = pattern.search(stripped)
                    if match is None:
                        continue
                    snippet = match.group(0).strip()
                    errors.append(f"{rel}:{lineno}: direct ops script entrypoint is forbidden (`{snippet}`)")
    return (0 if not errors else 1), sorted(set(errors))


def check_ops_scripts_are_data_only(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    manifests = repo_root / "ops" / "manifests"
    if not manifests.exists():
        return 1, ["missing ops/manifests directory"]
    for path in sorted(manifests.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        if path.suffix.lower() not in {".json", ".yaml", ".yml"}:
            errors.append(f"{rel}: manifests directory must contain json/yaml files only")
            continue
        if path.read_text(encoding="utf-8", errors="ignore").startswith("#!/"):
            errors.append(f"{rel}: data-only manifest must not be executable script")
    return (0 if not errors else 1), errors


def check_ops_shell_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    run_dir = repo_root / "ops" / "run"
    if not run_dir.exists():
        return 1, ["missing ops/run directory"]
    for path in sorted(run_dir.glob("*.sh")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if '. "$ROOT/ops/_lib/common.sh"' not in text:
            errors.append(f"{rel}: must source ops/_lib/common.sh")
        if "ops_entrypoint_start " not in text:
            errors.append(f"{rel}: missing ops_entrypoint_start")
        if "ops_version_guard " not in text and path.name != "prereqs.sh":
            errors.append(f"{rel}: missing ops_version_guard")
    return (0 if not errors else 1), errors


CHECKS: tuple[CheckDef, ...] = (
    CheckDef("ops.no_tracked_generated", "ops", "forbid tracked files in generated ops dirs", 800, check_ops_generated_tracked, fix_hint="Untrack generated ops files."),
    CheckDef("ops.no_tracked_timestamps", "ops", "forbid tracked timestamped paths", 1000, check_tracked_timestamp_paths, fix_hint="Remove timestamped tracked paths."),
    CheckDef("ops.committed_generated_hygiene", "ops", "validate deterministic committed generated assets", 1000, check_committed_generated_hygiene, fix_hint="Regenerate committed outputs deterministically."),
    CheckDef("ops.manifests_schema", "ops", "validate ops manifests against atlas.ops.manifest.v1 schema", 1000, check_ops_manifests_schema, fix_hint="Fix ops/manifests/*.json|*.yaml to satisfy atlas.ops.manifest.v1."),
    CheckDef("ops.no_direct_script_entrypoints", "ops", "forbid direct ops script entrypoints in docs/workflows/makefiles", 1000, check_ops_no_direct_script_entrypoints, fix_hint="Use ./bin/atlasctl ops ... or make wrappers, not ops/**/*.sh paths."),
    CheckDef("ops.scripts_are_data_only", "ops", "enforce ops/manifests data-only file policy", 1000, check_ops_scripts_are_data_only, fix_hint="Keep ops/manifests to json/yaml data only."),
    CheckDef("ops.shell_policy", "ops", "enforce shell runtime guard requirements for ops/run wrappers", 1000, check_ops_shell_policy, fix_hint="Source common.sh and call ops_entrypoint_start + ops_version_guard in ops/run/*.sh."),
)
