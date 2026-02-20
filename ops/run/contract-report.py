#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


@dataclass
class Check:
    name: str
    cmd: list[str]


CHECKS = [
    Check("generate-layer-contract", ["python3", "ops/_meta/generate_layer_contract.py"]),
    Check("check-layer-contract-drift", ["python3", "ops/_lint/check_layer_contract_drift.py"]),
    Check("check-layer-drift-static", ["python3", "scripts/layout/check_layer_drift.py"]),
    Check("validate-ops-contracts", ["python3", "scripts/layout/validate_ops_contracts.py"]),
    Check("check-literals", ["python3", "ops/_lint/no-layer-literals.py"]),
    Check("check-stack-literals", ["python3", "ops/_lint/no-stack-layer-literals.py"]),
    Check("check-no-hidden-defaults", ["python3", "scripts/layout/check_no_hidden_defaults.py"]),
    Check("check-k8s-layer-contract", ["ops/k8s/tests/checks/obs/test_layer_contract_render.sh"]),
    Check("check-live-layer-contract", ["ops/stack/tests/validate_live_snapshot.sh"]),
]


def run_check(check: Check, out_dir: Path) -> dict:
    started = datetime.now(timezone.utc).isoformat()
    proc = subprocess.run(
        check.cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    ended = datetime.now(timezone.utc).isoformat()
    log_path = out_dir / f"{check.name}.log"
    log_path.write_text(
        f"$ {' '.join(check.cmd)}\n\n{proc.stdout}{proc.stderr}",
        encoding="utf-8",
    )
    return {
        "name": check.name,
        "status": "pass" if proc.returncode == 0 else "fail",
        "exit_code": proc.returncode,
        "started_at": started,
        "ended_at": ended,
        "log": log_path.relative_to(ROOT).as_posix(),
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--run-id", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    out_path = ROOT / args.out
    out_path.parent.mkdir(parents=True, exist_ok=True)
    logs_dir = out_path.parent / "checks"
    logs_dir.mkdir(parents=True, exist_ok=True)

    results = [run_check(c, logs_dir) for c in CHECKS]
    failed = [r for r in results if r["status"] != "pass"]
    payload = {
        "schema_version": 1,
        "run_id": args.run_id,
        "status": "pass" if not failed else "fail",
        "contract": "ops/_meta/layer-contract.json",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "checks": results,
    }
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out_path.relative_to(ROOT).as_posix())
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
