from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from typing import Any

from atlasctl.core.context import RunContext
from atlasctl.core.runtime.paths import write_text_file
from atlasctl.commands.ops._shared.output import emit_ops_payload
from atlasctl.commands.ops.orchestrate._wrappers import artifact_base as _artifact_base
from atlasctl.commands.ops.tools import command_rendered, environment_summary, hash_inputs, invocation_report, preflight_tools, run_tool


def contracts_snapshot(ctx: RunContext, report_format: str, *, no_write: bool = False) -> int:
    checks = [
        ("generate-layer-contract", ["python3", "packages/atlasctl/src/atlasctl/commands/ops/meta/generate_layer_contract.py"]),
        ("check-layer-contract-drift", ["./bin/atlasctl", "check", "run", "--id", "checks_ops_script_ops_lint_check_layer_contract_drift_py", "--quiet"]),
        ("check-layer-drift-static", ["python3", "packages/atlasctl/src/atlasctl/layout_checks/check_layer_drift.py"]),
        ("validate-ops-contracts", ["python3", "packages/atlasctl/src/atlasctl/layout_checks/validate_ops_contracts.py"]),
        ("check-literals", ["python3", "packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_layer_literals.py"]),
        ("check-stack-literals", ["python3", "packages/atlasctl/src/atlasctl/commands/ops/lint/layout/no_stack_layer_literals.py"]),
        ("check-no-hidden-defaults", ["python3", "packages/atlasctl/src/atlasctl/layout_checks/check_no_hidden_defaults.py"]),
        ("check-k8s-layer-contract", ["python3", "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/obs/test_layer_contract_render.py"]),
        ("check-live-layer-contract", ["python3", "packages/atlasctl/src/atlasctl/commands/ops/stack/tests/validate_live_snapshot.py"]),
    ]
    out_dir = _artifact_base(ctx, "contracts") / "contracts"
    logs_dir = out_dir / "checks"
    if not no_write:
        out_dir.mkdir(parents=True, exist_ok=True)
        logs_dir.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, Any]] = []
    for name, cmd in checks:
        started = datetime.now(timezone.utc).isoformat()
        required = [cmd[0]] if cmd else []
        missing, _resolved = preflight_tools(required)
        if missing:
            code = 1
            combined_output = f"missing tools: {', '.join(missing)}"
            invocation_meta = {
                "tool": cmd[0] if cmd else "",
                "command_rendered": command_rendered(cmd),
                "timings": {"start_unix_s": 0.0, "end_unix_s": 0.0, "duration_ms": 0},
                "exit_code": 1,
                "stdout": "",
                "stderr": combined_output,
            }
        else:
            inv = run_tool(ctx, cmd)
            code = inv.code
            combined_output = inv.combined_output
            invocation_meta = invocation_report(inv)
        ended = datetime.now(timezone.utc).isoformat()
        log_path = logs_dir / f"{name}.log"
        if not no_write:
            write_text_file(log_path, f"$ {' '.join(cmd)}\n\n{combined_output}", encoding="utf-8")
        rows.append({
            "name": name,
            "status": "pass" if code == 0 else "fail",
            "exit_code": code,
            "started_at": started,
            "ended_at": ended,
            "command_rendered": command_rendered(cmd),
            "inputs_hash": hash_inputs(ctx.repo_root, ["ops/_meta/layer-contract.json"]),
            "environment_summary": environment_summary(ctx, [cmd[0]] if cmd else []),
            "timings": invocation_meta["timings"],
            "invocation": invocation_meta,
            "log": None if no_write else str(log_path.relative_to(ctx.repo_root)),
        })
    failed = [r for r in rows if r["status"] != "pass"]
    payload = {
        "schema_version": 1,
        "run_id": ctx.run_id,
        "status": "pass" if not failed else "fail",
        "contract": "ops/_meta/layer-contract.json",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "checks": rows,
    }
    if no_write:
        payload["no_write"] = True
    else:
        report_path = out_dir / "report.json"
        write_text_file(report_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if report_format == "json":
        emit_ops_payload(payload, report_format)
    else:
        emit_ops_payload(payload, "json", compact_json=False)
    return 0 if not failed else 1


def run_contracts_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    action = str(getattr(ns, "ops_contracts_cmd", "") or "snapshot").strip() or "snapshot"
    if action == "snapshot":
        return contracts_snapshot(ctx, getattr(ns, "report", "text"), no_write=bool(getattr(ns, "no_write", False)))
    return 2


__all__ = ["contracts_snapshot", "run_contracts_command"]
