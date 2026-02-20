from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path

from ..core.context import RunContext


def _venv_path(ctx: RunContext, override: str | None) -> Path:
    if override:
        raw = Path(override)
        return (ctx.repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()
    return (ctx.repo_root / "artifacts/bijux-atlas-scripts/venv/.venv").resolve()


def _lock_status(ctx: RunContext) -> str:
    lock = ctx.repo_root / "packages/bijux-atlas-scripts/requirements.lock.txt"
    return "ok" if lock.exists() and lock.stat().st_size > 0 else "missing"


def run_env_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    subcmd = getattr(ns, "env_cmd", None) or "info"
    if subcmd == "create":
        venv = _venv_path(ctx, ns.path)
        venv.parent.mkdir(parents=True, exist_ok=True)
        proc = subprocess.run(
            ["python3", "-m", "venv", str(venv)],
            cwd=ctx.repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "pass" if proc.returncode == 0 else "fail",
            "action": "env-create",
            "venv": str(venv),
            "lock_status": _lock_status(ctx),
        }
        print(json.dumps(payload, sort_keys=True) if ns.json else f"venv={venv} status={payload['status']}")
        if proc.returncode != 0 and proc.stderr:
            print(proc.stderr.strip())
        return proc.returncode
    if subcmd == "clean":
        root = (ctx.repo_root / "artifacts/bijux-atlas-scripts").resolve()
        removed: list[str] = []
        for name in (".pytest_cache", ".ruff_cache", ".mypy_cache", "run"):
            path = root / name
            if path.exists():
                shutil.rmtree(path, ignore_errors=True)
                removed.append(str(path))
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "pass",
            "action": "env-clean",
            "removed": removed,
        }
        print(json.dumps(payload, sort_keys=True) if ns.json else f"removed={len(removed)}")
        return 0

    # info
    venv = _venv_path(ctx, getattr(ns, "path", None))
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "action": "env-info",
        "python3": shutil.which("python3") or "missing",
        "venv": str(venv),
        "venv_exists": venv.exists(),
        "lock_status": _lock_status(ctx),
        "artifact_root": str(ctx.scripts_artifact_root),
    }
    print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
    return 0


def configure_env_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    env = sub.add_parser("env", help="manage isolated python environment and artifact-root policy")
    env.add_argument("--json", action="store_true", help="emit JSON output")
    env_sub = env.add_subparsers(dest="env_cmd", required=False)

    create = env_sub.add_parser("create", help="create venv in approved artifacts location")
    create.add_argument("--path", help="override venv path")
    create.add_argument("--json", action="store_true", help="emit JSON output")

    info = env_sub.add_parser("info", help="show python interpreter, venv, and lock status")
    info.add_argument("--path", help="override venv path")
    info.add_argument("--json", action="store_true", help="emit JSON output")

    clean = env_sub.add_parser("clean", help="clean package caches under artifacts root")
    clean.add_argument("--json", action="store_true", help="emit JSON output")
