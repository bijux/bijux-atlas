from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

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


def write_wrapper_artifacts(
    ctx: RunContext, area: str, action: str, cmd: list[str], code: int, output: str
) -> dict[str, Any]:
    out_dir = artifact_base(ctx, area)
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
        "command_rendered": command_rendered(cmd),
        "generated_at": started,
        "timings": {"start": started, "end": datetime.now(timezone.utc).isoformat()},
        "artifacts": {"run_log": str(run_log), "report": str(report_path)},
        "details": {
            "exit_code": code,
            "inputs_hash": hash_inputs(ctx.repo_root, []),
            "environment_summary": environment_summary(ctx, [cmd[0]] if cmd else []),
        },
    }
    write_text_file(report_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    validate_json(payload, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    return payload


def run_wrapped(ctx: RunContext, spec: OrchestrateSpec, report_format: str) -> int:
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
