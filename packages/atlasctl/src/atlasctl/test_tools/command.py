from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

from ..core.context import RunContext
from ..core.fs import write_json

_SMOKE_TESTS = (
    "packages/atlasctl/tests/cli/test_cli_smoke.py",
    "packages/atlasctl/tests/cli/test_cli_help_snapshot.py",
    "packages/atlasctl/tests/cli/test_cli_json_goldens.py",
)


def _smoke_command(ns: argparse.Namespace) -> list[str]:
    cmd = [sys.executable, "-m", "pytest", "-q", "-m", "unit", *_SMOKE_TESTS]
    if ns.pytest_args:
        cmd.extend(ns.pytest_args)
    return cmd


def _run_command(kind: str, ns: argparse.Namespace) -> list[str]:
    marker = {"unit": "unit", "integration": "integration"}[kind]
    cmd = [sys.executable, "-m", "pytest", "-q", "-m", marker]
    if ns.pytest_args:
        cmd.extend(ns.pytest_args)
    return cmd


def _isolation_env(ctx: RunContext, target_dir: str | None = None) -> tuple[dict[str, str], str]:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    resolved_target = target_dir or str(ctx.repo_root / "artifacts/isolate" / ctx.run_id / "atlasctl-test")
    os.makedirs(resolved_target, exist_ok=True)
    env["TMPDIR"] = resolved_target
    existing_opts = [token for token in env.get("PYTEST_ADDOPTS", "").split() if not token.startswith("--cache-dir=")]
    existing_opts.append(f"--basetemp={resolved_target}")
    env["PYTEST_ADDOPTS"] = " ".join(existing_opts).strip()
    return env, resolved_target


def run_test_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.test_cmd == "smoke":
        cmd = _smoke_command(ns)
        suite = "smoke"
        target_dir = None
    elif ns.test_cmd == "run":
        cmd = _run_command(ns.kind, ns)
        suite = ns.kind
        target_dir = ns.target_dir
    elif ns.test_cmd == "inventory":
        tests_root = ctx.repo_root / "packages/atlasctl/tests"
        by_domain: dict[str, list[str]] = {}
        for path in sorted(tests_root.rglob("test_*.py")):
            rel = path.relative_to(ctx.repo_root).as_posix()
            domain = path.parent.relative_to(tests_root).parts[0] if path.parent != tests_root else "root"
            by_domain.setdefault(domain, []).append(rel)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "total_tests": sum(len(rows) for rows in by_domain.values()),
            "domains": [{"domain": domain, "count": len(rows), "tests": rows} for domain, rows in sorted(by_domain.items())],
        }
        if ns.out_file:
            write_json(ctx, Path(ns.out_file), payload)
        print(json.dumps(payload, sort_keys=True) if (ns.json or ctx.output_format == "json") else f"tests={payload['total_tests']} domains={len(by_domain)}")
        return 0
    elif ns.test_cmd == "refresh-goldens":
        env, resolved_target = _isolation_env(ctx, ns.target_dir)
        cmd = [sys.executable, "-m", "atlasctl.cli", "--quiet", "gen", "goldens"]
        proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, capture_output=True, check=False)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if proc.returncode == 0 else "error",
            "command": cmd,
            "exit_code": proc.returncode,
            "target_dir": resolved_target,
        }
        print(json.dumps(payload, sort_keys=True) if (ns.json or ctx.output_format == "json") else f"refresh-goldens exit={proc.returncode}")
        if (proc.stdout or "").strip():
            print(proc.stdout.rstrip())
        if (proc.stderr or "").strip():
            print(proc.stderr.rstrip(), file=sys.stderr)
        return proc.returncode
    else:
        return 2
    env, resolved_target = _isolation_env(ctx, target_dir)
    proc = subprocess.run(
        cmd,
        cwd=ctx.repo_root,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if proc.returncode == 0 else "error",
        "suite": suite,
        "command": cmd,
        "exit_code": proc.returncode,
        "target_dir": resolved_target,
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"atlasctl test {suite}: {'pass' if proc.returncode == 0 else 'fail'} (exit={proc.returncode})")
    if (proc.stdout or "").strip():
        print(proc.stdout.rstrip())
    if (proc.stderr or "").strip():
        print(proc.stderr.rstrip(), file=sys.stderr)
    return proc.returncode


def configure_test_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("test", help="run atlasctl test suites")
    p_sub = p.add_subparsers(dest="test_cmd", required=True)
    smoke = p_sub.add_parser("smoke", help="run fast CLI smoke unit tests")
    smoke.add_argument("--json", action="store_true", help="emit machine-readable summary")
    smoke.add_argument("pytest_args", nargs="*", help="extra pytest args")
    run = p_sub.add_parser("run", help="run canonical pytest suites")
    run.add_argument("kind", choices=["unit", "integration"])
    run.add_argument("--json", action="store_true", help="emit machine-readable summary")
    run.add_argument("--target-dir", help="isolation directory for pytest temp/cache artifacts")
    run.add_argument("pytest_args", nargs="*", help="extra pytest args")
    inventory = p_sub.add_parser("inventory", help="report test inventory by domain")
    inventory.add_argument("--json", action="store_true", help="emit machine-readable summary")
    inventory.add_argument("--out-file", help="optional report output path")
    refresh = p_sub.add_parser("refresh-goldens", help="refresh goldens in isolated deterministic lane")
    refresh.add_argument("--json", action="store_true", help="emit machine-readable summary")
    refresh.add_argument("--target-dir", help="isolation directory for refresh artifacts")
