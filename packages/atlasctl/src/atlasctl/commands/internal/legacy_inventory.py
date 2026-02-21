from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

from ...core.context import RunContext


_LEGACY_CONCEPTS: tuple[dict[str, str], ...] = (
    {"module": "legacy/layout_shell/*", "status": "deleted", "reason": "replaced by shell/layout and repo checks"},
    {"module": "legacy/obs/*", "status": "deleted", "reason": "observability package is canonical"},
    {"module": "legacy/report/*", "status": "deleted", "reason": "reporting package is canonical"},
    {"module": "legacy/effects/*", "status": "deleted", "reason": "effect boundaries are enforced in checks/repo"},
    {"module": "legacy/subprocess.py", "status": "deleted", "reason": "core/exec.py is canonical process boundary"},
    {"module": "legacy/logging.py", "status": "deleted", "reason": "core.runtime.logging.py is canonical logging boundary"},
    {"module": "legacy/repo_checks_native*", "status": "moved", "reason": "moved into checks/repo and checks/repo/domains"},
    {"module": "legacy/ops_runtime*", "status": "moved", "reason": "moved into commands/ops and checks/layout/ops"},
    {"module": "legacy/docs_runtime*", "status": "moved", "reason": "moved into commands/docs runtime modules"},
)


def _classify(rel: str) -> tuple[str, str]:
    if rel.startswith("legacy/layout_shell/"):
        return "delete", "legacy shell layout checks were replaced by shell/layout and repo checks"
    if rel.startswith("legacy/obs/"):
        return "delete", "observability package is canonical for runtime and contract checks"
    if rel.startswith("legacy/report/"):
        return "delete", "reporting package is canonical for report assembly"
    if rel.startswith("legacy/effects/"):
        return "delete", "effect boundaries are enforced via checks/repo/enforcement/boundaries"
    if rel == "legacy/subprocess.py":
        return "delete", "core/exec.py is the only approved command execution boundary"
    if rel == "legacy/logging.py":
        return "delete", "core.runtime.logging.py is the only approved logging boundary"
    if rel.startswith("legacy/repo_checks_native"):
        return "move", "repo checks live under checks/repo and checks/repo/domains"
    if rel.startswith("legacy/ops_runtime"):
        return "move", "ops command runtime moved to commands/ops and checks/layout/ops"
    if rel.startswith("legacy/docs_runtime"):
        return "move", "docs command runtime moved to commands/docs and checks/docs"
    return "rewrite", "remaining legacy shim should be rewritten into canonical command/check modules"


def _reference_hits(repo_root: Path, pattern: str = "legacy") -> list[str]:
    cmd = ["rg", "-n", pattern, "packages/atlasctl/src/atlasctl", "packages/atlasctl/docs", "packages/atlasctl/tests"]
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode not in (0, 1):
        return []
    hits: list[str] = []
    for line in (proc.stdout or "").splitlines():
        rel = line.strip()
        if not rel:
            continue
        if re.search(r"(^|/)legacy_inventory\.py:", rel):
            continue
        hits.append(rel)
    return sorted(set(hits))


def run_legacy_inventory(ctx: RunContext, report: str) -> int:
    legacy_root = ctx.repo_root / "packages/atlasctl/src/atlasctl/legacy"
    rows: list[dict[str, str]] = []
    if legacy_root.exists():
        for path in sorted(legacy_root.rglob("*.py")):
            if "__pycache__" in path.parts:
                continue
            rel = path.relative_to(legacy_root.parent).as_posix()
            action, reason = _classify(rel)
            rows.append({"module": rel, "action": action, "reason": reason})
    references = _reference_hits(ctx.repo_root)

    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if len(rows) == 0 else "error",
        "action": "inventory",
        "run_id": ctx.run_id,
        "legacy_definition": "legacy means pre-1.0 compatibility modules/shims kept only to support migration and slated for deletion",
        "legacy_concepts": list(_LEGACY_CONCEPTS),
        "count": len(rows),
        "legacy_modules": rows,
        "reference_count": len(references),
        "references": references,
        "policy": "pre-1.0 release 0.1 hard gate: no legacy codepaths",
    }
    if report == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"legacy inventory: count={payload['count']}")
        for row in rows:
            print(f"- {row['module']} [{row['action']}] {row['reason']}")
    return 0 if len(rows) == 0 else 2
