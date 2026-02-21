from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ...contracts.output.base import build_output_base
from ..context import RunContext
from ..isolation import build_isolate_env
from .run_meta import write_run_meta

NEXTEST_TOML = "configs/nextest/nextest.toml"
DENY_CONFIG = "configs/security/deny.toml"
RUSTFMT_CONFIG = "configs/rust/rustfmt.toml"


@dataclass
class DevCargoParams:
    action: str
    all_tests: bool = False
    contracts_tests: bool = False
    json_output: bool = False
    verbose: bool = False


def _now_tag(action: str) -> str:
    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"atlasctl-{action}-{ts}-{os.getppid()}"


def _build_isolate_env(ctx: RunContext, action: str) -> dict[str, str]:
    env = build_isolate_env(
        repo_root=ctx.repo_root,
        git_sha=ctx.git_sha,
        prefix=f"atlasctl-{action}",
        tag=os.environ.get("ISO_TAG") or _now_tag(action),
        base_env=os.environ.copy(),
    )
    return env


def _atlasctl_cmd(*args: str, quiet: bool = False) -> list[str]:
    cmd = [sys.executable, "-m", "atlasctl.cli"]
    if quiet:
        cmd.append("--quiet")
    cmd.extend(args)
    return cmd


def _atlasctl_repo_check_cmd(*, quiet: bool, verbose: bool) -> list[str]:
    cmd = _atlasctl_cmd("check", "run", "--group", "repo", quiet=quiet)
    if verbose:
        cmd.append("--verbose")
    else:
        cmd.append("--quiet")
    return cmd


def _run(
    *,
    ctx: RunContext,
    env: dict[str, str],
    cmd: list[str],
    steps: list[dict[str, Any]],
    verbose: bool,
    quiet: bool,
) -> int:
    step = {"command": " ".join(cmd)}
    if (not quiet) and (not verbose):
        print(f"run: {step['command']}")
    if verbose:
        proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, check=False)
        step["exit_code"] = proc.returncode
        steps.append(step)
        if not quiet:
            print(f"result: {'pass' if proc.returncode == 0 else 'fail'}")
        return proc.returncode
    proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, capture_output=True, check=False)
    step["exit_code"] = proc.returncode
    step["stdout"] = proc.stdout or ""
    step["stderr"] = proc.stderr or ""
    steps.append(step)
    if not quiet:
        print(f"result: {'pass' if proc.returncode == 0 else 'fail'}")
    return proc.returncode


def _run_test_cleanup(env: dict[str, str]) -> None:
    for rel in ("target/nextest", "target"):
        path = Path(rel)
        if not path.exists():
            continue
        if rel == "target/nextest":
            for child in path.rglob("*"):
                if child.is_file():
                    child.unlink(missing_ok=True)
        for child in sorted(path.rglob("*"), reverse=True):
            if child.is_dir():
                try:
                    child.rmdir()
                except OSError:
                    pass
        try:
            path.rmdir()
        except OSError:
            pass


def run_dev_cargo(ctx: RunContext, params: DevCargoParams) -> int:
    env = _build_isolate_env(ctx, params.action)
    ctx.require_isolate(env)
    steps: list[dict[str, Any]] = []
    failures: list[str] = []
    action = params.action
    cargo_jobs = env.get("JOBS", "")
    if cargo_jobs:
        env["CARGO_BUILD_JOBS"] = cargo_jobs

    emit_progress = (not params.json_output) and (not ctx.quiet)

    def run_cmd(cmd: list[str]) -> bool:
        code = _run(ctx=ctx, env=env, cmd=cmd, steps=steps, verbose=params.verbose, quiet=not emit_progress)
        if code != 0:
            failures.append(" ".join(cmd))
            return False
        return True

    if action == "fmt":
        run_cmd(["cargo", "fmt", "--all", "--", "--check", "--config-path", RUSTFMT_CONFIG]) and run_cmd(
            _atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose)
        )
    elif action == "lint":
        run_cmd(["cargo", "fmt", "--all", "--", "--check", "--config-path", RUSTFMT_CONFIG]) and run_cmd(
            _atlasctl_cmd("policies", "check", "--fail-fast", quiet=ctx.quiet)
        ) and run_cmd(_atlasctl_cmd("check", "no-direct-bash-invocations", quiet=ctx.quiet)) and run_cmd(
            _atlasctl_cmd("check", "no-direct-python-invocations", quiet=ctx.quiet)
        ) and run_cmd(_atlasctl_cmd("check", "scripts-surface-docs-drift", quiet=ctx.quiet)) and run_cmd(
            _atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose)
        ) and run_cmd(_atlasctl_cmd("docs", "link-check", "--report", "text", quiet=ctx.quiet)) and run_cmd(
            [
                "cargo",
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]
        )
    elif action == "check":
        run_cmd(["cargo", "check", "--workspace", "--all-targets"]) and run_cmd(
            _atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose)
        )
    elif action == "test":
        if params.contracts_tests:
            run_cmd(["cargo", "test", "-p", "bijux-atlas-server", "--test", "observability_contract"]) and run_cmd(
                _atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose)
            )
        else:
            profile = env.get("NEXTEST_PROFILE", "ci")
            cmd = [
                "cargo",
                "nextest",
                "run",
                "--workspace",
                "--all-targets",
                "--profile",
                profile,
                "--config-file",
                NEXTEST_TOML,
            ]
            if params.all_tests:
                cmd.extend(["--run-ignored", "all"])
            run_cmd(["cargo", "nextest", "--version"]) and run_cmd(cmd)
            _run_test_cleanup(env)
            if not failures:
                run_cmd(_atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose))
    elif action == "coverage":
        profile = env.get("NEXTEST_PROFILE", "ci")
        output = Path(env.get("ISO_ROOT", str(ctx.repo_root / "artifacts" / "isolate" / "local"))) / "coverage" / "lcov.info"
        output.parent.mkdir(parents=True, exist_ok=True)
        run_cmd(["cargo", "llvm-cov", "--version"]) and run_cmd(
            [
                "cargo",
                "llvm-cov",
                "nextest",
                "--workspace",
                "--profile",
                profile,
                "--config-file",
                NEXTEST_TOML,
                "--lcov",
                "--output-path",
                str(output),
            ]
        ) and run_cmd(_atlasctl_repo_check_cmd(quiet=ctx.quiet, verbose=params.verbose))
    elif action == "audit":
        if shutil.which("cargo") is None:
            failures.append("cargo not found")
        else:
            deny_check = subprocess.run(
                ["cargo", "+stable", "deny", "--version"],
                cwd=ctx.repo_root,
                env=env,
                text=True,
                capture_output=not params.verbose,
                check=False,
            )
            steps.append({"command": "cargo +stable deny --version", "exit_code": deny_check.returncode})
            if deny_check.returncode != 0:
                run_cmd(["cargo", "+stable", "install", "cargo-deny", "--locked"])
            if not failures:
                run_cmd(["cargo", "+stable", "deny", "check", "--config", DENY_CONFIG])
    else:
        failures.append(f"unsupported action: {action}")

    ok = not failures
    meta = {
        "action": action,
        "all_tests": params.all_tests,
        "contracts_tests": params.contracts_tests,
        "steps": steps,
        "isolate_root": env.get("ISO_ROOT", ""),
        "isolate_tag": env.get("ISO_TAG", ""),
    }
    out_dir = ctx.repo_root / "artifacts" / "evidence" / "dev" / ctx.run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    meta_path = write_run_meta(ctx, out_dir, lane=action)
    meta["run_meta"] = str(meta_path)
    payload = build_output_base(run_id=ctx.run_id, ok=ok, errors=failures, meta=meta, version=2)
    payload["status"] = "ok" if ok else "error"
    (out_dir / f"dev-{action}.report.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if params.json_output:
        print(json.dumps(payload, sort_keys=True))
    elif ok:
        print(f"dev {action}: pass")
    else:
        print(f"dev {action}: fail")
        if not params.verbose:
            for step in steps:
                if step.get("exit_code") != 0:
                    print(f"- failed: {step['command']}")
                    break
    return 0 if ok else 1
