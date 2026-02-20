from __future__ import annotations

import os
import runpy
import sys
from pathlib import Path

from .run_context import RunContext


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def run_legacy_script(script_path: str, args: list[str], ctx: RunContext) -> int:
    root = repo_root()
    script = (root / script_path).resolve()
    if not script.exists():
        raise SystemExit(f"script not found: {script_path}")
    os.environ.setdefault("RUN_ID", ctx.run_id)
    os.environ.setdefault("EVIDENCE_ROOT", str(ctx.evidence_root))
    os.environ.setdefault("PROFILE", ctx.profile)
    old_argv = sys.argv[:]
    old_path = sys.path[:]
    old_cwd = Path.cwd()
    sys.argv = [str(script), *args]
    sys.path.insert(0, str(script.parent))
    os.chdir(root)
    try:
        runpy.run_path(str(script), run_name="__main__")
        return 0
    except SystemExit as exc:
        return int(exc.code) if isinstance(exc.code, int) else 1
    finally:
        sys.argv = old_argv
        sys.path[:] = old_path
        os.chdir(old_cwd)
