from __future__ import annotations

import argparse
import subprocess

from ..core.context import RunContext


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ci_cmd == "scripts":
        proc = subprocess.run(["make", "-s", "scripts-check"], cwd=ctx.repo_root, text=True, check=False)
        return proc.returncode
    return 2


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ci", help="ci command group")
    ci_sub = p.add_subparsers(dest="ci_cmd", required=True)
    ci_sub.add_parser("scripts", help="run scripts ci checks")
