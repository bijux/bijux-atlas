from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from atlasctl.core.context import RunContext
from atlasctl.core.schema.schema_utils import validate_json

from atlasctl.commands.ops.orchestrate._wrappers import OrchestrateSpec, emit_payload, run_wrapped


def run_scenario_from_manifest(
    ctx: RunContext, report_format: str, manifest: str, scenario: str, *, no_write: bool = False
) -> int:
    manifest_path = (ctx.repo_root / manifest).resolve()
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    validate_json(payload, ctx.repo_root / "configs/ops/scenarios.schema.json")
    scenarios = payload.get("scenarios", {})
    item = scenarios.get(scenario)
    if not isinstance(item, dict):
        fail: dict[str, Any] = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "fail",
            "run_id": ctx.run_id,
            "area": "run",
            "action": scenario,
            "details": f"scenario `{scenario}` missing in {manifest}",
        }
        emit_payload(fail, report_format)
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
        emit_payload(fail, report_format)
        return 2
    return run_wrapped(
        ctx, OrchestrateSpec("run", scenario, [str(x) for x in cmd]), report_format, dry_run=bool(no_write)
    )
