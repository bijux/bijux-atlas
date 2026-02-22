from __future__ import annotations

import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from ..commands.policies.runtime.culprits import budget_suite
from ..checks.domains.policies.make.enforcement import collect_bypass_inventory


def _tool_version(cmd: list[str]) -> str:
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip()
        return out.splitlines()[0] if out else "unknown"
    except Exception:
        return "missing"


def build_report(ctx: RunContext) -> dict[str, object]:
    toolchain_ok = (ctx.repo_root / "python-toolchain.toml").exists()
    repo_ok = (ctx.repo_root / ".git").exists() and (ctx.repo_root / "makefiles").is_dir()
    write_roots_ok = ctx.scripts_artifact_root.as_posix().find("artifacts/atlasctl") != -1
    python_env_ok = shutil.which("python3") is not None
    budget = budget_suite(ctx.repo_root)
    tree_health = {
        "status": budget.get("status", "unknown"),
        "report_count": len(budget.get("reports", [])),
        "failing_metrics": [r.get("metric") for r in budget.get("reports", []) if r.get("status") == "fail"],
        "warning_metrics": [r.get("metric") for r in budget.get("reports", []) if r.get("status") == "warn"],
    }
    bypass_inventory = collect_bypass_inventory(ctx.repo_root)
    return {
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "repo_root": str(ctx.repo_root),
        "evidence_root": str(ctx.evidence_root),
        "scripts_artifact_root": str(ctx.scripts_artifact_root),
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "tools": {
            "python3": shutil.which("python3") or "missing",
            "make": _tool_version(["make", "-v"]),
            "git": _tool_version(["git", "--version"]),
        },
        "pins": {
            "tool_versions": str(ctx.repo_root / "configs/ops/tool-versions.json"),
            "python_lock": str(ctx.repo_root / "packages/atlasctl/requirements.lock.txt"),
        },
        "env": {
            "RUN_ID": os.environ.get("RUN_ID", ""),
            "EVIDENCE_ROOT": os.environ.get("EVIDENCE_ROOT", ""),
            "BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT": os.environ.get("BIJUX_ATLAS_SCRIPTS_ARTIFACT_ROOT", ""),
            "PROFILE": os.environ.get("PROFILE", ""),
        },
        "checks": {
            "python_env_ok": python_env_ok,
            "repo_root_ok": repo_ok,
            "toolchain_ok": toolchain_ok,
            "write_roots_ok": write_roots_ok,
        },
        "tree_health": tree_health,
        "bypass_inventory": {
            "file_count": len(bypass_inventory.get("files", [])) if isinstance(bypass_inventory.get("files"), list) else 0,
            "entry_count": int(bypass_inventory.get("entry_count", 0)),
        },
    }


def run_doctor(ctx: RunContext, as_json: bool, out_file: str | None) -> int:
    report = build_report(ctx)
    if out_file:
        out = ensure_evidence_path(ctx, Path(out_file))
        out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if as_json:
        print(json.dumps(report, sort_keys=True))
    else:
        print(f"run_id={ctx.run_id}")
        print(f"profile={ctx.profile}")
        print(f"repo_root={ctx.repo_root}")
        print(f"evidence_root={ctx.evidence_root}")
        print(f"python={report['python']}")
    return 0
