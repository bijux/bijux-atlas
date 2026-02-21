from __future__ import annotations

import argparse
import json
from pathlib import Path

from ..core.context import RunContext


def _classify(rel: str) -> tuple[str, str]:
    if rel.startswith("legacy/layout_shell/"):
        return "delete", "legacy shell layout checks were replaced by checks/layout/shell and repo checks"
    if rel.startswith("legacy/obs/"):
        return "delete", "observability package is canonical for runtime and contract checks"
    if rel.startswith("legacy/report/"):
        return "delete", "reporting package is canonical for report assembly"
    if rel.startswith("legacy/effects/"):
        return "delete", "effect boundaries are enforced via checks/repo/enforcement/boundaries"
    if rel == "legacy/subprocess.py":
        return "delete", "core/exec.py is the only approved command execution boundary"
    if rel == "legacy/logging.py":
        return "delete", "core/logging.py is the only approved logging boundary"
    if rel.startswith("legacy/repo_checks_native"):
        return "move", "repo checks live under checks/repo and checks/repo/domains"
    if rel.startswith("legacy/ops_runtime"):
        return "move", "ops command runtime moved to commands/ops and checks/layout/ops"
    if rel.startswith("legacy/docs_runtime"):
        return "move", "docs command runtime moved to commands/docs and checks/docs"
    return "rewrite", "remaining legacy shim should be rewritten into canonical command/check modules"


def run_legacy_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    legacy_root = ctx.repo_root / "packages/atlasctl/src/atlasctl/legacy"
    rows: list[dict[str, str]] = []
    if legacy_root.exists():
        for path in sorted(legacy_root.rglob("*.py")):
            if "__pycache__" in path.parts:
                continue
            rel = path.relative_to(legacy_root.parent).as_posix()
            action, reason = _classify(rel)
            rows.append({"module": rel, "action": action, "reason": reason})

    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "action": "inventory",
        "run_id": ctx.run_id,
        "count": len(rows),
        "legacy_modules": rows,
        "policy": "pre-1.0: legacy code must be deleted, not preserved",
    }
    if ns.report == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"legacy inventory: count={payload['count']}")
        for row in rows:
            print(f"- {row['module']} [{row['action']}] {row['reason']}")
    return 0


def configure_legacy_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("legacy", help="legacy inventory and removal tracking")
    p.add_argument("legacy_cmd", choices=["inventory"])
    p.add_argument("--report", choices=["text", "json"], default="text")
