from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from ..context import RunContext


def _tool_version(ctx: RunContext, cmd: list[str]) -> str:
    try:
        proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    except OSError:
        return "missing"
    if proc.returncode != 0:
        return "unavailable"
    out = (proc.stdout or proc.stderr or "").strip()
    return out.splitlines()[0] if out else "unknown"


def write_run_meta(ctx: RunContext, out_dir: Path, *, lane: str) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "run_id": ctx.run_id,
        "lane": lane,
        "git_sha": ctx.git_sha,
        "git_dirty": ctx.git_dirty,
        "host": {"platform": sys.platform, "python": sys.version.split()[0]},
        "tool_versions": {
            "cargo": _tool_version(ctx, ["cargo", "--version"]),
            "nextest": _tool_version(ctx, ["cargo", "nextest", "--version"]),
            "llvm_cov": _tool_version(ctx, ["cargo", "llvm-cov", "--version"]),
        },
    }
    out_path = out_dir / "run.meta.json"
    out_path.write_text(__import__("json").dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out_path
