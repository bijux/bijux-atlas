from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from ....core.context import RunContext
from ....core.isolation import build_isolate_env, clean_isolate_roots, require_isolate_env


def _venv_path(ctx: RunContext, override: str | None) -> Path:
    if override:
        raw = Path(override)
        return (ctx.repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()
    return (ctx.repo_root / "artifacts/atlasctl/venv/.venv").resolve()


def _lock_status(ctx: RunContext) -> str:
    lock = ctx.repo_root / "packages/atlasctl/requirements.lock.txt"
    return "ok" if lock.exists() and lock.stat().st_size > 0 else "missing"


def _load_scripts_retention(ctx: RunContext) -> tuple[int, int]:
    cfg = ctx.repo_root / "configs/ops/scripts-artifact-retention.json"
    if not cfg.exists():
        return 14, 20
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    return int(payload.get("scripts_retention_days", 14)), int(payload.get("scripts_keep_last_runs", 20))


def clean_scripts_artifacts(ctx: RunContext, older_than_days: int | None = None) -> dict[str, object]:
    root = (ctx.repo_root / "artifacts/atlasctl").resolve()
    removed: list[str] = []
    if not root.exists():
        return {
            "schema_version": 1,
            "tool": "atlasctl",
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
        "tool": "atlasctl",
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
            "tool": "atlasctl",
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
    if subcmd == "require-isolate":
        ok, message = require_isolate_env(os.environ.copy())
        if not ok:
            print(f"isolate-required: {message}")
            return 1
        print("isolate-required: OK")
        return 0
    if subcmd == "isolate":
        env = build_isolate_env(
            repo_root=ctx.repo_root,
            git_sha=ctx.git_sha or "nogit",
            prefix="atlasctl-env-isolate",
            tag=getattr(ns, "tag", None) or os.environ.get("ISO_TAG", ""),
            base_env=os.environ.copy(),
        )
        if ns.print_root:
            print(env["ISO_ROOT"])
            return 0
        if ns.print_tag:
            print(env["ISO_TAG"])
            return 0
        if ns.print_env:
            keys = ("ISO_TAG", "ISO_RUN_ID", "ISO_ROOT", "CARGO_TARGET_DIR", "CARGO_HOME", "TMPDIR", "TMP", "TEMP", "TZ", "LC_ALL")
            for key in keys:
                print(f"{key}={env[key]}")
            return 0
        if ns.clean:
            removed = clean_isolate_roots(
                ctx.repo_root,
                older_than_days=int(getattr(ns, "older_than_days", 14)),
                keep_last=int(getattr(ns, "keep_last", 20)),
            )
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "action": "isolate-clean",
                "removed": removed,
            }
            print(json.dumps(payload, sort_keys=True) if ns.json else f"removed={len(removed)}")
            return 0
        if not ns.exec_cmd:
            print("usage: atlasctl env isolate [options] <command> [args...]")
            return 2
        iso_root = Path(env["ISO_ROOT"])
        if ns.require_clean and not ns.reuse and iso_root.exists():
            print(f"isolate: ISO_ROOT already exists: {iso_root}")
            return 1
        (iso_root / "target").mkdir(parents=True, exist_ok=True)
        (iso_root / "cargo-home").mkdir(parents=True, exist_ok=True)
        (iso_root / "tmp").mkdir(parents=True, exist_ok=True)
        proc = subprocess.run(ns.exec_cmd, cwd=ctx.repo_root, env=env, check=False)
        return proc.returncode

    # info
    venv = _venv_path(ctx, getattr(ns, "path", None))
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
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
    req = env_sub.add_parser("require-isolate", help="assert isolate env variables are set and rooted under artifacts/isolate")
    req.add_argument("--json", action="store_true", help="emit JSON output")
    iso = env_sub.add_parser("isolate", help="run command inside deterministic isolate runtime directories")
    iso.add_argument("--print-root", action="store_true")
    iso.add_argument("--print-env", action="store_true")
    iso.add_argument("--print-tag", action="store_true")
    iso.add_argument("--tag")
    iso.add_argument("--require-clean", action="store_true")
    iso.add_argument("--reuse", action="store_true")
    iso.add_argument("--clean", action="store_true", help="remove stale isolate roots under artifacts/isolate")
    iso.add_argument("--older-than-days", type=int, default=14, help="age threshold for --clean")
    iso.add_argument("--keep-last", type=int, default=20, help="retain newest N roots for --clean")
    iso.add_argument("exec_cmd", nargs=argparse.REMAINDER)
    iso.add_argument("--json", action="store_true", help="emit JSON output")
