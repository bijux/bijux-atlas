from __future__ import annotations

import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

from .run_context import RunContext


def _tool_version(cmd: list[str]) -> str:
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip()
        return out.splitlines()[0] if out else "unknown"
    except Exception:
        return "missing"


def build_report(ctx: RunContext) -> dict[str, object]:
    root = Path(__file__).resolve().parents[4]
    return {
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        "evidence_root": str(ctx.evidence_root),
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "tools": {
            "python3": shutil.which("python3") or "missing",
            "make": _tool_version(["make", "-v"]),
            "git": _tool_version(["git", "--version"]),
        },
        "pins": {
            "tool_versions": str(root / "configs/ops/tool-versions.json"),
            "python_lock": str(root / "tools/bijux_atlas_scripts/requirements.lock.txt"),
        },
        "env": {
            "RUN_ID": os.environ.get("RUN_ID", ""),
            "EVIDENCE_ROOT": os.environ.get("EVIDENCE_ROOT", ""),
            "PROFILE": os.environ.get("PROFILE", ""),
        },
    }


def run_doctor(ctx: RunContext, as_json: bool, out_file: str | None) -> int:
    report = build_report(ctx)
    if out_file:
        out = Path(out_file)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if as_json:
        print(json.dumps(report, sort_keys=True))
    else:
        print(f"run_id={ctx.run_id}")
        print(f"profile={ctx.profile}")
        print(f"evidence_root={ctx.evidence_root}")
        print(f"python={report['python']}")
    return 0
