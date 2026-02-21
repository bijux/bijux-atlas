from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

from ..contracts.command import run_contracts_command
from ..core.context import RunContext

SELF_CLI = ["python3", "-m", "atlasctl.cli"]


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


def _run_capture(ctx: RunContext, args: list[str]) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ctx.repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


def _write(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def _generate_goldens(ctx: RunContext) -> tuple[int, dict[str, str]]:
    targets: dict[str, list[str]] = {
        "help.json.golden": ["--quiet", "help", "--json"],
        "commands.json.golden": ["--quiet", "commands", "--json"],
        "surface.json.golden": ["--quiet", "surface", "--json"],
        "explain.check.json.golden": ["--quiet", "--json", "explain", "check"],
        "check-list.json.golden": ["--quiet", "--json", "check", "list"],
        "cli_help_snapshot.txt": ["--help"],
    }
    out_dir = ctx.repo_root / "packages/atlasctl/tests/goldens"
    written: dict[str, str] = {}
    for name, cmd in targets.items():
        proc = _run_capture(ctx, cmd)
        if proc.returncode != 0:
            msg = proc.stderr.strip() or proc.stdout.strip() or f"failed to generate {name}"
            return 1, {"error": f"{name}: {msg}"}
        content = proc.stdout if name.endswith(".txt") else proc.stdout.strip() + "\n"
        path = out_dir / name
        _write(path, content)
        written[name] = path.relative_to(ctx.repo_root).as_posix()
    commands = json.loads((out_dir / "help.json.golden").read_text(encoding="utf-8"))["commands"]
    command_names = "\n".join(row["name"] for row in commands) + "\n"
    cmd_path = out_dir / "cli_help_commands.expected.txt"
    _write(cmd_path, command_names)
    written["cli_help_commands.expected.txt"] = cmd_path.relative_to(ctx.repo_root).as_posix()
    return 0, written


def run_gen_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.gen_cmd
    if sub == "contracts":
        return run_contracts_command(
            ctx,
            argparse.Namespace(contracts_cmd="generate", report="text", generators=["artifacts"]),
        )
    if sub == "openapi":
        return run_contracts_command(
            ctx,
            argparse.Namespace(contracts_cmd="generate", report="text", generators=["openapi"]),
        )
    if sub == "ops-surface":
        return _run(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/generate_ops_surface_meta.py"])
    if sub == "make-targets":
        return _run(ctx, [*SELF_CLI, "make", "inventory"])
    if sub == "surface":
        return _run(ctx, ["python3", "-m", "atlasctl.cli", "docs", "generate-repo-surface", "--report", "text"])
    if sub == "scripting-surface":
        out = ctx.repo_root / "docs/_generated/scripts-surface.md"
        lines = [
            "# Scripts Surface",
            "",
            "Generated file. Do not edit manually.",
            "",
            "## scripts/bin",
            "",
        ]
        for p in sorted((ctx.repo_root / "scripts/bin").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        lines.extend(["", "## checks", ""])
        for p in sorted((ctx.repo_root / "scripts/areas/check").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        lines.extend(["", "## root bin shims", ""])
        for p in sorted((ctx.repo_root / "bin").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        out.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(json.dumps({"status": "pass", "file": str(out)}, sort_keys=True))
        return 0
    if sub == "goldens":
        code, written = _generate_goldens(ctx)
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok" if code == 0 else "error", "written": written}
        print(json.dumps(payload, sort_keys=True))
        return code
    return 2


def configure_gen_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("gen", help="generation commands mapped from scripts/areas")
    p_sub = p.add_subparsers(dest="gen_cmd", required=True)
    p_sub.add_parser("contracts", help="generate contracts artifacts")
    p_sub.add_parser("openapi", help="generate openapi snapshot and telemetry artifacts")
    p_sub.add_parser("goldens", help="generate test golden snapshots under packages/atlasctl/tests/goldens")
    p_sub.add_parser("ops-surface", help="generate ops surface metadata")
    p_sub.add_parser("make-targets", help="generate make targets inventory artifacts")
    p_sub.add_parser("surface", help="generate repo public surface artifacts")
    p_sub.add_parser("scripting-surface", help="generate scripts/CLI surface artifacts")
