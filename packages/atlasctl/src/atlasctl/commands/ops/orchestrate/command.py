from __future__ import annotations

import argparse
import json
import socket
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path
from ....core.process import run_command
from ....core.runtime.paths import write_text_file
from ....core.schema.schema_utils import validate_json


@dataclass(frozen=True)
class OrchestrateSpec:
    area: str
    action: str
    cmd: list[str]


def _artifact_base(ctx: RunContext, area: str) -> Path:
    return ensure_evidence_path(ctx, ctx.evidence_root / area / ctx.run_id)


def _write_wrapper_artifacts(ctx: RunContext, area: str, action: str, cmd: list[str], code: int, output: str) -> dict[str, Any]:
    out_dir = _artifact_base(ctx, area)
    started = datetime.now(timezone.utc).isoformat()
    run_log = out_dir / "run.log"
    report_path = out_dir / "report.json"
    write_text_file(run_log, output + ("\n" if output and not output.endswith("\n") else ""), encoding="utf-8")
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass" if code == 0 else "fail",
        "run_id": ctx.run_id,
        "area": area,
        "action": action,
        "command": " ".join(cmd),
        "generated_at": started,
        "artifacts": {
            "run_log": str(run_log),
            "report": str(report_path),
        },
        "details": {"exit_code": code},
    }
    write_text_file(report_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    validate_json(payload, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    return payload


def _emit(payload: dict[str, Any], report_format: str) -> None:
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            f"{payload['area']}:{payload['action']} status={payload['status']} run_id={payload['run_id']} "
            f"log={payload['artifacts']['run_log']}"
        )


def _run_wrapped(ctx: RunContext, spec: OrchestrateSpec, report_format: str) -> int:
    result = run_command(spec.cmd, ctx.repo_root, ctx=ctx)
    output = result.combined_output
    payload = _write_wrapper_artifacts(ctx, spec.area, spec.action, spec.cmd, result.code, output)
    _emit(payload, report_format)
    return result.code


def _ports_show(ctx: RunContext, report_format: str) -> int:
    ports_cfg = json.loads((ctx.repo_root / "configs/ops/ports.json").read_text(encoding="utf-8"))
    payload: dict[str, Any] = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "ports",
        "action": "show",
        "details": ports_cfg,
    }
    _emit(payload, report_format)
    return 0


def _reserve_ephemeral_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        s.listen(1)
        return int(s.getsockname()[1])


def _ports_reserve(ctx: RunContext, report_format: str, name: str, port: int | None) -> int:
    chosen = int(port) if port is not None else _reserve_ephemeral_port()
    out_dir = _artifact_base(ctx, "ports")
    reservation = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "ports",
        "action": "reserve",
        "details": {"name": name, "port": chosen},
    }
    validate_json(reservation, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    write_text_file(out_dir / "report.json", json.dumps(reservation, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_text_file(out_dir / "run.log", f"reserved {name}={chosen}\n", encoding="utf-8")
    _emit(reservation, report_format)
    return 0


def _cleanup(ctx: RunContext, report_format: str, older_than_days: int) -> int:
    now = datetime.now(timezone.utc).timestamp()
    root = ctx.evidence_root
    removed: list[str] = []
    if root.exists():
        for path in sorted(root.rglob("*")):
            if not path.is_dir():
                continue
            age_days = (now - path.stat().st_mtime) / 86400.0
            if age_days >= float(older_than_days):
                removed.append(str(path))
    # Remove deepest paths first.
    for item in sorted(removed, key=lambda p: p.count("/"), reverse=True):
        p = Path(item)
        try:
            p.rmdir()
        except OSError:
            continue
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "cleanup",
        "action": "gc",
        "details": {"older_than_days": older_than_days, "removed_dirs": sorted(removed)},
    }
    validate_json(payload, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    _emit(payload, report_format)
    return 0


def _run_manifest(ctx: RunContext, report_format: str, manifest: str, scenario: str) -> int:
    manifest_path = (ctx.repo_root / manifest).resolve()
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    scenarios = payload.get("scenarios", {})
    item = scenarios.get(scenario)
    if not isinstance(item, dict):
        fail = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "fail",
            "run_id": ctx.run_id,
            "area": "run",
            "action": scenario,
            "details": f"scenario `{scenario}` missing in {manifest}",
        }
        _emit(fail, report_format)
        return 2
    cmd = item.get("command")
    if not isinstance(cmd, list) or not cmd:
        fail = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "fail",
            "run_id": ctx.run_id,
            "area": "run",
            "action": scenario,
            "details": f"scenario `{scenario}` has invalid command",
        }
        _emit(fail, report_format)
        return 2
    return _run_wrapped(ctx, OrchestrateSpec("run", scenario, [str(x) for x in cmd]), report_format)


def run_orchestrate_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.cmd == "ports":
        if ns.ports_cmd == "show":
            return _ports_show(ctx, ns.report)
        return _ports_reserve(ctx, ns.report, ns.name, ns.port)

    if ns.cmd == "artifacts":
        return _run_wrapped(ctx, OrchestrateSpec("artifacts", "open", ["bash", "ops/run/artifacts-open.sh"]), ns.report)
    if ns.cmd == "k8s":
        mapping = {
            "render": ["helm", "template", "atlas", "ops/chart"],
            "install": ["bash", "ops/run/deploy-atlas.sh"],
            "uninstall": ["bash", "ops/run/undeploy.sh"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("k8s", ns.k8s_cmd, mapping[ns.k8s_cmd]), ns.report)
    if ns.cmd == "stack":
        mapping = {
            "up": ["bash", "ops/run/stack-up.sh"],
            "down": ["bash", "ops/run/stack-down.sh"],
            "reset": ["bash", "-lc", "ops/run/stack-down.sh && ops/run/stack-up.sh"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("stack", ns.stack_cmd, mapping[ns.stack_cmd]), ns.report)
    if ns.cmd == "obs":
        mapping = {
            "up": ["bash", "ops/run/obs-up.sh"],
            "verify": ["bash", "ops/run/obs-verify.sh"],
            "down": ["bash", "ops/run/obs-validate.sh", "--mode", "down"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("obs", ns.obs_cmd, mapping[ns.obs_cmd]), ns.report)
    if ns.cmd == "load":
        mapping = {
            "smoke": ["bash", "ops/run/load-smoke.sh"],
            "suite": ["bash", "ops/run/load-suite.sh"],
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
            "smoke": ["bash", "ops/run/e2e-smoke.sh"],
            "realdata": ["bash", "ops/run/e2e.sh", "--suite", "realdata"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("e2e", ns.e2e_cmd, mapping[ns.e2e_cmd]), ns.report)
    if ns.cmd == "datasets":
        mapping = {
            "verify": ["bash", "ops/run/datasets-verify.sh"],
            "fetch": ["bash", "ops/run/warm.sh"],
            "pin": ["python3", "packages/atlasctl/src/atlasctl/datasets/build_manifest_lock.py"],
        }
        return _run_wrapped(ctx, OrchestrateSpec("datasets", ns.datasets_cmd, mapping[ns.datasets_cmd]), ns.report)
    if ns.cmd == "contracts-snapshot":
        return _run_wrapped(
            ctx,
            OrchestrateSpec("contracts", "snapshot", ["bash", "ops/run/contract-check.sh"]),
            ns.report,
        )
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
