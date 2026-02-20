from __future__ import annotations

import argparse
import subprocess

from ..core.context import RunContext


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, check=False)
    return proc.returncode


def run_check_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.check_cmd
    if sub == "layout":
        return _run(ctx, ["python3", "scripts/areas/layout/check_layer_drift.py"])
    if sub == "make":
        return _run(ctx, ["python3", "scripts/areas/layout/check_makefiles_contract.py"])
    if sub == "docs":
        return _run(ctx, ["python3", "scripts/areas/docs/check_make_targets_documented.py"])
    if sub == "configs":
        return _run(ctx, ["python3", "scripts/areas/configs/validate_configs_schemas.py"])
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
