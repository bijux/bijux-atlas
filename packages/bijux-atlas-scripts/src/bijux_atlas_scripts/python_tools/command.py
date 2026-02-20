from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from pathlib import Path

from ..core.context import RunContext
from ..exit_codes import ERR_PREREQ, ERR_USER


def _venv_path(ctx: RunContext, override: str | None) -> Path:
    if override:
        raw = Path(override)
        path = (ctx.repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()
    else:
        path = (ctx.scripts_root / "venv/.venv").resolve()
    if ctx.scripts_root not in path.parents and path != ctx.scripts_root:
        raise ValueError(f"venv path must be under {ctx.scripts_root}")
    return path


def _problem_paths_from_status(status_lines: list[str]) -> list[str]:
    bad: list[str] = []
    patterns = (".pytest_cache/", ".ruff_cache/", ".mypy_cache/", "__pycache__/", ".pyc", ".pyo")
    for line in status_lines:
        if not line.strip():
            continue
        path = line[3:] if len(line) > 3 else line
        if any(p in path for p in patterns):
            bad.append(path)
            continue
        if "/.venv/" in path or path.startswith(".venv/"):
            if not path.startswith("artifacts/bijux-atlas-scripts/venv/"):
                bad.append(path)
    return sorted(set(bad))


def _status_lines(ctx: RunContext) -> list[str]:
    proc = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ctx.repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    return proc.stdout.splitlines() if proc.returncode == 0 else []


def _cmd_lint(ctx: RunContext, as_json: bool) -> int:
    bad = _problem_paths_from_status(_status_lines(ctx))
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "pass" if not bad else "fail",
        "check": "python-cache-tracking",
        "violations": bad,
    }
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        if bad:
            print("python cache tracking check failed:")
            for p in bad:
                print(f"- {p}")
        else:
            print("python cache tracking check passed")
    return 0 if not bad else ERR_USER


def _cmd_clean(ctx: RunContext, as_json: bool) -> int:
    root = ctx.scripts_root
    removed: list[str] = []
    for rel in ("cache", "pycache", "mypy", "ruff", "pytest", "pip", "pytest-tmp", "hypothesis", "run"):
        path = root / rel
        if path.exists():
            shutil.rmtree(path, ignore_errors=True)
            removed.append(str(path))
    payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "removed": sorted(removed)}
    print(json.dumps(payload, sort_keys=True) if as_json else f"removed={len(removed)}")
    return 0


def _cmd_venv_create(ctx: RunContext, path_arg: str | None, as_json: bool) -> int:
    try:
        venv = _venv_path(ctx, path_arg)
    except ValueError as exc:
        print(str(exc))
        return ERR_USER
    venv.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(["python3", "-m", "venv", str(venv)], cwd=ctx.repo_root, text=True, check=False)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if proc.returncode == 0 else "fail",
        "venv": str(venv),
    }
    print(json.dumps(payload, sort_keys=True) if as_json else f"venv={venv} status={payload['status']}")
    return proc.returncode


def _cmd_venv_run(ctx: RunContext, path_arg: str | None, cmd: list[str], as_json: bool) -> int:
    if not cmd:
        print("usage: atlasctl python venv run -- <cmd>")
        return ERR_USER
    try:
        venv = _venv_path(ctx, path_arg)
    except ValueError as exc:
        print(str(exc))
        return ERR_USER
    py = venv / "bin/python"
    if not py.exists():
        print(f"missing venv interpreter: {py}")
        return ERR_PREREQ
    env = os.environ.copy()
    env["VIRTUAL_ENV"] = str(venv)
    env["PATH"] = f"{venv / 'bin'}:{env.get('PATH', '')}"
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False, env=env)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if proc.returncode == 0 else "fail",
        "venv": str(venv),
        "command": cmd,
        "exit_code": proc.returncode,
    }
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    return proc.returncode


def run_python_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    as_json = bool(getattr(ns, "json", False)) or ctx.output_format == "json"
    if ns.python_cmd == "lint":
        return _cmd_lint(ctx, as_json)
    if ns.python_cmd == "clean":
        return _cmd_clean(ctx, as_json)
    if ns.python_cmd == "venv":
        if ns.venv_cmd == "create":
            return _cmd_venv_create(ctx, ns.path, as_json)
        if ns.venv_cmd == "run":
            return _cmd_venv_run(ctx, ns.path, ns.cmd, as_json)
    return ERR_USER


def configure_python_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("python", help="python runtime isolation helpers")
    p.add_argument("--json", action="store_true", help="emit JSON output")
    p_sub = p.add_subparsers(dest="python_cmd", required=True)
    p_sub.add_parser("lint", help="validate no cache dirs are tracked")
    p_sub.add_parser("clean", help="clean python artifact dirs under artifacts/bijux-atlas-scripts")

    venv = p_sub.add_parser("venv", help="venv lifecycle helpers")
    venv_sub = venv.add_subparsers(dest="venv_cmd", required=True)
    create = venv_sub.add_parser("create", help="create venv under artifacts root")
    create.add_argument("--path", help="override venv path under artifacts root")
    run = venv_sub.add_parser("run", help="run command with standardized venv")
    run.add_argument("--path", help="override venv path under artifacts root")
    run.add_argument("cmd", nargs=argparse.REMAINDER)
