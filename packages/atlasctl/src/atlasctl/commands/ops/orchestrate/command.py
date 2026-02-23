from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ....core.context import RunContext
from ..artifacts.command import artifacts_open as _artifacts_open_cmd, cleanup_gc as _cleanup_gc_cmd
from ..contracts.command import contracts_snapshot as _contracts_snapshot_cmd
from ..ports.command import ports_reserve as _ports_reserve_cmd, ports_show as _ports_show_cmd
from ..scenario.command import run_scenario_from_manifest as _run_scenario_from_manifest_cmd
from ..tools import command_rendered, invocation_report, run_tool
from ._wrappers import OrchestrateSpec, artifact_base as _artifact_base, emit_payload as _emit, run_wrapped as _run_wrapped


def _ports_show(ctx: RunContext, report_format: str) -> int:
    return _ports_show_cmd(ctx, report_format)


def _ports_reserve(ctx: RunContext, report_format: str, name: str, port: int | None) -> int:
    return _ports_reserve_cmd(ctx, report_format, name, port)


def _cleanup(ctx: RunContext, report_format: str, older_than_days: int) -> int:
    return _cleanup_gc_cmd(ctx, report_format, older_than_days)


def _artifacts_open(ctx: RunContext, report_format: str) -> int:
    return _artifacts_open_cmd(ctx, report_format)


def _run_manifest(ctx: RunContext, report_format: str, manifest: str, scenario: str) -> int:
    return _run_scenario_from_manifest_cmd(ctx, report_format, manifest, scenario)


def _stack_reset(ctx: RunContext, report_format: str) -> int:
    out_dir = _artifact_base(ctx, "stack")
    steps = [
        ("down", ["./bin/atlasctl", "ops", "stack", "--report", "text", "down"]),
        ("up", ["./bin/atlasctl", "ops", "stack", "--report", "text", "up"]),
    ]
    rows: list[dict[str, Any]] = []
    final_code = 0
    for name, cmd in steps:
        inv = run_tool(ctx, cmd)
        log_path = out_dir / f"reset-{name}.log"
        write_text_file(log_path, inv.combined_output, encoding="utf-8")
        rows.append(
            {
                "name": name,
                "status": "pass" if inv.code == 0 else "fail",
                "command_rendered": command_rendered(cmd),
                "timings": invocation_report(inv)["timings"],
                "exit_code": inv.code,
                "log": str(log_path.relative_to(ctx.repo_root)),
            }
        )
        if inv.code != 0:
            final_code = inv.code
            break
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass" if final_code == 0 else "fail",
        "run_id": ctx.run_id,
        "area": "stack",
        "action": "reset",
        "steps": rows,
        "artifacts": {"report": str((out_dir / "report.json").relative_to(ctx.repo_root))},
    }
    write_text_file(out_dir / "report.json", json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    _emit(payload, report_format)
    return final_code


def run_orchestrate_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.cmd == "ports":
        if ns.ports_cmd == "show":
            return _ports_show(ctx, ns.report)
        return _ports_reserve(ctx, ns.report, ns.name, ns.port)

    if ns.cmd == "artifacts":
        return _artifacts_open(ctx, ns.report)
    if ns.cmd == "k8s":
        mapping = {
            "render": ["helm", "template", "atlas", "ops/chart"],
            "install": ["./bin/atlasctl", "ops", "deploy", "--report", "text", "apply"],
            "uninstall": ["./bin/atlasctl", "ops", "deploy", "--report", "text", "rollback"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("k8s", ns.k8s_cmd, mapping[ns.k8s_cmd]), ns.report)
    if ns.cmd == "stack":
        mapping = {
            "up": ["./bin/atlasctl", "ops", "stack", "--report", "text", "up"],
            "down": ["./bin/atlasctl", "ops", "stack", "--report", "text", "down"],
        }
        if ns.stack_cmd == "reset":
            return _stack_reset(ctx, ns.report)
        return _run_wrapped(ctx, OrchestrateSpec("stack", ns.stack_cmd, mapping[ns.stack_cmd]), ns.report)
    if ns.cmd == "obs":
        mapping = {
            "up": ["./bin/atlasctl", "ops", "obs", "--report", "text", "up"],
            "verify": ["./bin/atlasctl", "ops", "obs", "--report", "text", "verify"],
            "down": ["./bin/atlasctl", "ops", "obs", "--report", "text", "validate"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("obs", ns.obs_cmd, mapping[ns.obs_cmd]), ns.report)
    if ns.cmd == "load":
        mapping = {
            "smoke": ["make", "ops-load-smoke"],
            "suite": ["./bin/atlasctl", "ops", "load", "--report", "text", "run"],
            "baseline-compare": [
                "python3",
                "packages/atlasctl/src/atlasctl/load/baseline/compare_runs.py",
            ],
            "baseline-update": [
                "python3",
                "packages/atlasctl/src/atlasctl/load/baseline/update_baseline.py",
            ],
        }
        return _run_wrapped(ctx, OrchestrateSpec("load", ns.load_cmd, mapping[ns.load_cmd]), ns.report)
    if ns.cmd == "e2e":
        mapping = {
            "smoke": ["./bin/atlasctl", "ops", "e2e", "--report", "text", "run", "--suite", "smoke"],
            "realdata": ["./bin/atlasctl", "ops", "e2e", "--report", "text", "run", "--suite", "realdata"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("e2e", ns.e2e_cmd, mapping[ns.e2e_cmd]), ns.report)
    if ns.cmd == "datasets":
        mapping = {
            "verify": ["./bin/atlasctl", "ops", "datasets", "--report", "text", "verify"],
            "fetch": ["./bin/atlasctl", "ops", "warm", "--report", "text", "--mode", "warmup"],
            "pin": ["python3", "packages/atlasctl/src/atlasctl/datasets/build_manifest_lock.py"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("datasets", ns.datasets_cmd, mapping[ns.datasets_cmd]), ns.report)
    if ns.cmd == "contracts-snapshot":
        return _contracts_snapshot_cmd(ctx, ns.report)
    if ns.cmd == "cleanup":
        return _cleanup(ctx, ns.report, ns.older_than)
    if ns.cmd == "scenario":
        return _run_manifest(ctx, ns.report, ns.manifest, ns.scenario)
    return 2


def configure_orchestrate_parsers(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    ports = sub.add_parser("ports", help="port registry and reservation commands")
    ports_sub = ports.add_subparsers(dest="ports_cmd", required=True)
    p_show = ports_sub.add_parser("show", help="show SSOT port registry")
    p_show.add_argument("--report", choices=["text", "json"], default="text")
    p_reserve = ports_sub.add_parser("reserve", help="reserve an ephemeral port for current run")
    p_reserve.add_argument("--name", required=True)
    p_reserve.add_argument("--port", type=int)
    p_reserve.add_argument("--report", choices=["text", "json"], default="text")

    artifacts = sub.add_parser("artifacts", help="artifacts helpers")
    artifacts_sub = artifacts.add_subparsers(dest="artifacts_cmd", required=True)
    a_open = artifacts_sub.add_parser("open", help="open latest artifacts")
    a_open.add_argument("--report", choices=["text", "json"], default="text")

    k8s = sub.add_parser("k8s", help="k8s wrappers")
    k8s_sub = k8s.add_subparsers(dest="k8s_cmd", required=True)
    for name in ("render", "install", "uninstall"):
        cmd = k8s_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    stack = sub.add_parser("stack", help="stack lifecycle wrappers")
    stack_sub = stack.add_subparsers(dest="stack_cmd", required=True)
    for name in ("up", "down", "reset"):
        cmd = stack_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    obs = sub.add_parser("obs", help="observability wrappers")
    obs_sub = obs.add_subparsers(dest="obs_cmd", required=True)
    for name in ("up", "verify", "down"):
        cmd = obs_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    load = sub.add_parser("load", help="load wrappers")
    load_sub = load.add_subparsers(dest="load_cmd", required=True)
    for name in ("smoke", "suite", "baseline-compare", "baseline-update"):
        cmd = load_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    e2e = sub.add_parser("e2e", help="e2e wrappers")
    e2e_sub = e2e.add_subparsers(dest="e2e_cmd", required=True)
    for name in ("smoke", "realdata"):
        cmd = e2e_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    datasets = sub.add_parser("datasets", help="dataset wrappers")
    datasets_sub = datasets.add_subparsers(dest="datasets_cmd", required=True)
    for name in ("verify", "fetch", "pin"):
        cmd = datasets_sub.add_parser(name)
        cmd.add_argument("--report", choices=["text", "json"], default="text")

    cleanup = sub.add_parser("cleanup", help="cleanup artifacts by retention policy")
    cleanup.add_argument("--older-than", type=int, default=14)
    cleanup.add_argument("--report", choices=["text", "json"], default="text")

    scenario = sub.add_parser("scenario", help="run scenario from manifest")
    scenario.add_argument("--manifest", default="ops/e2e/suites/suites.json")
    scenario.add_argument("--scenario", required=True)
    scenario.add_argument("--report", choices=["text", "json"], default="text")

    contracts_snapshot = sub.add_parser("contracts-snapshot", help="run live contracts snapshot check")
    contracts_snapshot.add_argument("--report", choices=["text", "json"], default="text")
