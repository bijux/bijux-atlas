from __future__ import annotations

import sys
from pathlib import Path

from ..context import RunContext
from ..exec import run as process_run
from ..fs import write_json


def _tool_version(ctx: RunContext, cmd: list[str]) -> str:
    try:
        proc = process_run(cmd, cwd=ctx.repo_root, text=True, capture_output=True)
    except OSError:
        return "missing"
    if proc.returncode != 0:
        return "unavailable"
    out = (proc.stdout or proc.stderr or "").strip()
    return out.splitlines()[0] if out else "unknown"


def write_run_meta(ctx: RunContext, out_dir: Path, *, lane: str) -> Path:
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
    return write_json(ctx, out_dir / "run.meta.json", payload)


__all__ = ["write_run_meta"]
