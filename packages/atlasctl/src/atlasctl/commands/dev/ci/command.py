from __future__ import annotations

import argparse

from ....core.context import RunContext
from ....core.effects.dev_ci import LANE_FILTERS, run_ci_command as run_ci_effect


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    return run_ci_effect(ctx, ns)


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ci", help="ci command group")
    p.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    ci_sub = p.add_subparsers(dest="ci_cmd", required=True)
    ls = ci_sub.add_parser("list", help="list canonical CI lanes")
    ls.add_argument("--json", action="store_true", help="emit JSON output")
    ls.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    ci_sub.add_parser("scripts", help="run scripts ci checks")
    run = ci_sub.add_parser("run", help="run canonical CI suite locally")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    run.add_argument("--out-dir", help="output directory for CI artifacts")
    run.add_argument("--lane", action="append", choices=sorted(LANE_FILTERS.keys()), help="restrict suite run to a logical lane")
    mode = run.add_mutually_exclusive_group()
    mode.add_argument("--fail-fast", action="store_true", help="stop at first failing suite step")
    mode.add_argument("--keep-going", action="store_true", help="continue through all suite steps (default)")
    run.add_argument("--no-isolate", action="store_true", help="debug only: skip isolate wrapper around suite execution")
    run.add_argument("--explain", action="store_true", help="print planned CI run steps without executing")
    run.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    report = ci_sub.add_parser("report", help="show CI run report paths")
    report.add_argument("--latest", action="store_true", help="show the latest CI run report")
    report.add_argument("--json", action="store_true", help="emit JSON output")
    report.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    for name in (
        "all",
        "init",
        "artifacts",
        "release",
        "release-all",
        "fast",
        "contracts",
        "docs",
        "ops",
        "init-iso-dirs",
        "init-tmp",
        "dependency-lock-refresh",
        "release-compat-matrix-verify",
        "release-build-artifacts",
        "release-notes-render",
        "release-publish-gh",
        "cosign-sign",
        "cosign-verify",
        "chart-package-release",
        "reproducible-verify",
        "security-advisory-render",
        "governance-check",
        "pr",
        "nightly",
    ):
        sp = ci_sub.add_parser(name, help=f"run ci action: {name}")
        sp.add_argument("--json", action="store_true", help="emit JSON output")
        sp.add_argument("--verbose", action="store_true", help="show underlying tool command output")
