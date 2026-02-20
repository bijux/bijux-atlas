from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from datetime import datetime, timezone
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


def _load_scripts_retention(ctx: RunContext) -> tuple[int, int]:
    cfg = ctx.repo_root / "configs/ops/scripts-artifact-retention.json"
    if not cfg.exists():
        return 14, 20
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    return int(payload.get("scripts_retention_days", 14)), int(payload.get("scripts_keep_last_runs", 20))


def clean_scripts_artifacts(ctx: RunContext, older_than_days: int | None = None) -> dict[str, object]:
    root = (ctx.repo_root / "artifacts/bijux-atlas-scripts").resolve()
    removed: list[str] = []
    if not root.exists():
        return {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "pass",
            "action": "clean",
            "removed": removed,
        }
    days, keep_last = _load_scripts_retention(ctx)
    if older_than_days is not None:
        days = int(older_than_days)
    cutoff_ts = datetime.now(timezone.utc).timestamp() - (days * 86400)
    runs = sorted((root / "run").glob("*"), key=lambda p: p.stat().st_mtime if p.exists() else 0.0, reverse=True)
    keep = {p.resolve() for p in runs[:keep_last]}
    for path in runs:
        if path.resolve() in keep:
            continue
        if path.stat().st_mtime <= cutoff_ts:
            shutil.rmtree(path, ignore_errors=True)
            removed.append(str(path))
    for name in (".pytest_cache", ".ruff_cache", ".mypy_cache", ".hypothesis"):
        path = root / name
        if path.exists():
            shutil.rmtree(path, ignore_errors=True)
            removed.append(str(path))
    return {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "action": "clean",
        "removed": sorted(removed),
        "retention_days": days,
        "keep_last_runs": keep_last,
    }


def run_env_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    subcmd = getattr(ns, "env_cmd", None) or "info"
    env_payload = {
        "XDG_CACHE_HOME": str((ctx.scripts_root / "cache").resolve()),
        "PYTHONPYCACHEPREFIX": str((ctx.scripts_root / "pycache").resolve()),
        "MYPY_CACHE_DIR": str((ctx.scripts_root / "mypy").resolve()),
        "RUFF_CACHE_DIR": str((ctx.scripts_root / "ruff").resolve()),
        "PIP_CACHE_DIR": str((ctx.scripts_root / "pip").resolve()),
        "PYTEST_ADDOPTS": f"--cache-dir={(ctx.scripts_root / 'pytest').resolve()}",
    }
    if subcmd == "print":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "action": "env-print",
            "env": env_payload,
        }
        print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
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
        payload = clean_scripts_artifacts(ctx)
        removed = payload.get("removed", [])
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
    prn = env_sub.add_parser("print", help="print standardized python/cache environment values")
    prn.add_argument("--json", action="store_true", help="emit JSON output")
