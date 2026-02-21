from __future__ import annotations

import argparse
import json
import subprocess

from ..contracts.command import run_contracts_command
from ..core.context import RunContext

SELF_CLI = ["python3", "-m", "atlasctl.cli"]


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


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
    return 2


def configure_gen_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("gen", help="generation commands mapped from scripts/areas")
    p_sub = p.add_subparsers(dest="gen_cmd", required=True)
    p_sub.add_parser("contracts", help="generate contracts artifacts")
    p_sub.add_parser("openapi", help="generate openapi snapshot and telemetry artifacts")
    p_sub.add_parser("ops-surface", help="generate ops surface metadata")
    p_sub.add_parser("make-targets", help="generate make targets inventory artifacts")
    p_sub.add_parser("surface", help="generate repo public surface artifacts")
    p_sub.add_parser("scripting-surface", help="generate scripts/CLI surface artifacts")
