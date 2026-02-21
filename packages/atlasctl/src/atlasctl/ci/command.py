from __future__ import annotations

import argparse
import json
import subprocess

from ..core.context import RunContext


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.ci_cmd == "scripts":
        proc = subprocess.run(["make", "-s", "scripts-check"], cwd=ctx.repo_root, text=True, check=False)
        return proc.returncode
    if ns.ci_cmd == "run":
        proc = subprocess.run(
            ["python3", "-m", "atlasctl.cli", "--quiet", "--format", "json", "suite", "run", "ci"],
            cwd=ctx.repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
        if ns.json or ctx.output_format == "json":
            print(proc.stdout.strip() if proc.stdout.strip() else json.dumps({"status": "fail", "stderr": proc.stderr.strip()}, sort_keys=True))
        elif proc.returncode == 0:
            print("ci run: pass (suite ci)")
        else:
            print("ci run: fail (suite ci)")
        return proc.returncode
    return 2


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ci", help="ci command group")
    ci_sub = p.add_subparsers(dest="ci_cmd", required=True)
    ci_sub.add_parser("scripts", help="run scripts ci checks")
    run = ci_sub.add_parser("run", help="run canonical CI suite locally")
    run.add_argument("--json", action="store_true", help="emit JSON output")
