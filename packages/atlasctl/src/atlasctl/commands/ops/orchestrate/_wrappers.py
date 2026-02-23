from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
import time

from atlasctl.core.context import RunContext
from atlasctl.core.fs import ensure_evidence_path
from atlasctl.core.runtime.paths import write_text_file
from atlasctl.core.schema.schema_utils import validate_json

from ..tools import command_rendered, environment_summary, hash_inputs, invocation_report, preflight_tools, run_tool


@dataclass(frozen=True)
class OrchestrateSpec:
    area: str
    action: str
    cmd: list[str]


def artifact_base(ctx: RunContext, area: str) -> Path:
    return ensure_evidence_path(ctx, ctx.evidence_root / area / ctx.run_id)


def emit_payload(payload: dict[str, Any], report_format: str) -> None:
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            f"{payload['area']}:{payload['action']} status={payload['status']} run_id={payload['run_id']} "
            f"log={payload.get('artifacts', {}).get('run_log', '-')}"
        )


def _failure_taxonomy(code: int, output: str) -> dict[str, Any]:
    if code == 0:
        return {"kind": "none", "exit_code": 0}
    text = (output or "").lower()
    if "missing tools:" in text or "not found" in text:
        kind = "missing-tool"
    elif "schema" in text or "invalid" in text:
        kind = "invalid-config"
    elif "contract" in text:
        kind = "contract-fail"
    else:
        kind = "exit-code"
    return {"kind": kind, "exit_code": int(code)}


def write_wrapper_artifacts(
    ctx: RunContext, area: str, action: str, cmd: list[str], code: int, output: str
) -> dict[str, Any]:
    out_dir = artifact_base(ctx, area)
    t0 = time.time()
    started = datetime.now(timezone.utc).isoformat()
    run_log = out_dir / "run.log"
    report_path = out_dir / "report.json"
    artifact_index_path = out_dir / "artifact-index.json"
    write_text_file(run_log, output + ("\n" if output and not output.endswith("\n") else ""), encoding="utf-8")
    artifact_index = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "area": area,
        "action": action,
        "artifacts": [
            {"name": "run_log", "path": str(run_log), "kind": "text/log"},
            {"name": "report", "path": str(report_path), "kind": "application/json"},
        ],
    }
    write_text_file(artifact_index_path, json.dumps(artifact_index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    t1 = time.time()
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass" if code == 0 else "fail",
        "run_id": ctx.run_id,
        "area": area,
        "action": action,
        "command": " ".join(cmd),
        "command_rendered": command_rendered(cmd),
        "generated_at": started,
        "timings": {
            "start": started,
            "end": datetime.now(timezone.utc).isoformat(),
            "duration_ms": int((t1 - t0) * 1000),
        },
        "artifacts": {
            "run_log": str(run_log),
            "report": str(report_path),
            "artifact_index": str(artifact_index_path),
        },
        "details": {
            "exit_code": code,
            "failure": _failure_taxonomy(code, output),
            "inputs_hash": hash_inputs(ctx.repo_root, []),
            "environment_summary": environment_summary(ctx, [cmd[0]] if cmd else []),
        },
    }
    write_text_file(report_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    validate_json(payload, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    return payload


def run_wrapped(ctx: RunContext, spec: OrchestrateSpec, report_format: str, *, dry_run: bool = False) -> int:
    if dry_run:
        out_dir = ctx.evidence_root / spec.area / ctx.run_id
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "pass",
            "run_id": ctx.run_id,
            "area": spec.area,
            "action": spec.action,
            "command": " ".join(spec.cmd),
            "command_rendered": command_rendered(spec.cmd),
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "timings": {"start": "", "end": "", "duration_ms": 0},
            "artifacts": {
                "run_log": str((out_dir / "run.log")),
                "report": str((out_dir / "report.json")),
                "artifact_index": str((out_dir / "artifact-index.json")),
            },
            "details": {
                "exit_code": 0,
                "failure": {"kind": "none", "exit_code": 0},
                "inputs_hash": hash_inputs(ctx.repo_root, []),
                "environment_summary": environment_summary(ctx, [spec.cmd[0]] if spec.cmd else []),
                "dry_run": True,
            },
        }
        emit_payload(payload, report_format)
        return 0
    missing, _resolved = preflight_tools([spec.cmd[0]] if spec.cmd else [])
    if missing:
        payload = write_wrapper_artifacts(ctx, spec.area, spec.action, spec.cmd, 1, f"missing tools: {', '.join(missing)}")
        emit_payload(payload, report_format)
        return 1
    inv = run_tool(ctx, spec.cmd)
    payload = write_wrapper_artifacts(ctx, spec.area, spec.action, spec.cmd, inv.code, inv.combined_output)
    payload["details"]["invocation"] = invocation_report(inv)
    emit_payload(payload, report_format)
    return inv.code
