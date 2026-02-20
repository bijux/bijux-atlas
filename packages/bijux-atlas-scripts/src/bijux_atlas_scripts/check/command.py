from __future__ import annotations

import argparse
import json
import subprocess

from ..core.context import RunContext
from ..lint.runner import run_suite


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.check_cmd
    if sub in {"make", "docs", "configs"}:
        suite_name = {"make": "makefiles", "docs": "docs", "configs": "configs"}[sub]
        code, payload = run_suite(ctx.repo_root, suite_name, fail_fast=False)
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"check {sub}: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        return code
    if sub == "layout":
        return _run(ctx, ["python3", "scripts/areas/layout/check_layer_drift.py"])
    if sub == "obs":
        return _run(
            ctx,
            [
                "python3",
                "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_metrics_contract.py",
            ],
        )
    if sub == "stack-report":
        return _run(ctx, ["python3", "scripts/areas/public/stack/validate_stack_report.py"])
    if sub == "cli-help":
        return _run(ctx, ["python3", "scripts/areas/check/check-script-help.py"])
    if sub == "ownership":
        return _run(ctx, ["python3", "scripts/areas/check/check-script-ownership.py"])
    if sub == "duplicate-script-names":
        return _run(ctx, ["python3", "scripts/areas/check/check_duplicate_script_names.py"])
    return 2


def configure_check_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("check", help="area-based checks mapped from scripts/areas")
    p_sub = p.add_subparsers(dest="check_cmd", required=True)
    p_sub.add_parser("layout", help="run layout checks")
    p_sub.add_parser("make", help="run makefile checks")
    p_sub.add_parser("docs", help="run docs checks")
    p_sub.add_parser("configs", help="run configs checks")
    p_sub.add_parser("obs", help="run observability checks")
    p_sub.add_parser("stack-report", help="validate stack report contracts")
    p_sub.add_parser("cli-help", help="validate script/CLI help coverage")
    p_sub.add_parser("ownership", help="validate script ownership coverage")
    p_sub.add_parser("duplicate-script-names", help="validate duplicate script names")
