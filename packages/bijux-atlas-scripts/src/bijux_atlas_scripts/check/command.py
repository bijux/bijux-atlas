from __future__ import annotations

import argparse
import json
import subprocess

from ..core.context import RunContext
from ..lint.runner import run_suite
from .native import (
    check_committed_generated_hygiene,
    check_layout_contract,
    check_make_command_allowlist,
    check_duplicate_script_names,
    check_make_forbidden_paths,
    check_make_help,
    check_make_scripts_references,
    check_ops_generated_tracked,
    check_no_xtask_refs,
    check_script_help,
    check_script_ownership,
    check_tracked_timestamp_paths,
)


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
        code, errors = check_layout_contract(ctx.repo_root)
        if errors:
            print("layout contract failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("layout contract passed")
        return code
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
        code, errors = check_script_help(ctx.repo_root)
        if errors:
            print("script help contract failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script help contract passed")
        return code
    if sub == "ownership":
        code, errors = check_script_ownership(ctx.repo_root)
        if errors:
            print("script ownership coverage failed:")
            for err in errors:
                print(f"- {err}")
        else:
            print("script ownership coverage passed")
        return code
    if sub == "duplicate-script-names":
        code, errors = check_duplicate_script_names(ctx.repo_root)
        if errors:
            print("duplicate dash/underscore script names detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no duplicate script names")
        return code
    if sub == "make-scripts-refs":
        code, errors = check_make_scripts_references(ctx.repo_root)
        if errors:
            print("make scripts reference policy failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("make scripts reference policy passed")
        return code
    if sub == "make-help":
        code, errors = check_make_help(ctx.repo_root)
        if errors:
            for err in errors:
                print(err)
        else:
            print("make help output is deterministic")
        return code
    if sub == "forbidden-paths":
        code_script_refs, script_ref_errors = check_make_scripts_references(ctx.repo_root)
        code_paths, errors = check_make_forbidden_paths(ctx.repo_root)
        if script_ref_errors:
            print("make scripts reference policy failed:")
            for err in script_ref_errors[:200]:
                print(f"- {err}")
        if errors:
            print("forbidden make recipe paths detected:")
            for err in errors:
                print(f"- {err}")
        if code_script_refs == 0 and code_paths == 0:
            print("make forbidden path checks passed")
            return 0
        return 1
    if sub == "no-xtask":
        code, errors = check_no_xtask_refs(ctx.repo_root)
        if errors:
            print("xtask references detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no xtask references detected")
        return code
    if sub == "ops-generated-tracked":
        code, errors = check_ops_generated_tracked(ctx.repo_root)
        if errors:
            print("tracked files detected under ops/_generated:")
            for err in errors:
                print(f"- {err}")
        else:
            print("ops/_generated has no tracked files")
        return code
    if sub == "tracked-timestamps":
        code, errors = check_tracked_timestamp_paths(ctx.repo_root)
        if errors:
            print("tracked timestamp-like paths detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("no tracked timestamp-like paths detected")
        return code
    if sub == "committed-generated-hygiene":
        code, errors = check_committed_generated_hygiene(ctx.repo_root)
        if errors:
            print("committed generated hygiene violations detected:")
            for err in errors:
                print(f"- {err}")
        else:
            print("committed generated directories contain deterministic assets only")
        return code
    if sub == "make-command-allowlist":
        code, errors = check_make_command_allowlist(ctx.repo_root)
        if errors:
            print("make command allowlist check failed:")
            for err in errors[:200]:
                print(f"- {err}")
        else:
            print("make command allowlist check passed")
        return code
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
    p_sub.add_parser("make-scripts-refs", help="validate no makefile references to scripts paths")
    p_sub.add_parser("make-help", help="validate deterministic make help output")
    p_sub.add_parser("forbidden-paths", help="forbid scripts/xtask/tools direct recipe paths")
    p_sub.add_parser("no-xtask", help="forbid xtask references outside ADR history")
    p_sub.add_parser("ops-generated-tracked", help="fail if ops/_generated contains tracked files")
    p_sub.add_parser("tracked-timestamps", help="fail if tracked paths contain timestamp-like directories")
    p_sub.add_parser(
        "committed-generated-hygiene",
        help="fail on runtime/timestamped artifacts in committed generated directories",
    )
    p_sub.add_parser("make-command-allowlist", help="enforce direct make recipe command allowlist")
