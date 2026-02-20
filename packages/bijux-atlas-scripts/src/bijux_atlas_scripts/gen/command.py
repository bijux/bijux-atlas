from __future__ import annotations

import argparse
import subprocess

from ..core.context import RunContext


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


def run_gen_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.gen_cmd
    if sub == "contracts":
        return _run(ctx, ["python3", "scripts/areas/contracts/generate_contract_artifacts.py"])
    if sub == "openapi":
        return _run(ctx, ["bash", "scripts/areas/internal/openapi-generate.sh"])
    if sub == "ops-surface":
        return _run(ctx, ["python3", "scripts/areas/layout/generate_ops_surface_meta.py"])
    return 2


def configure_gen_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("gen", help="generation commands mapped from scripts/areas")
    p_sub = p.add_subparsers(dest="gen_cmd", required=True)
    p_sub.add_parser("contracts", help="generate contracts artifacts")
    p_sub.add_parser("openapi", help="generate openapi snapshot and telemetry artifacts")
    p_sub.add_parser("ops-surface", help="generate ops surface metadata")
