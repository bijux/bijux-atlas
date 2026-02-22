#!/usr/bin/env python3
# Purpose: run one gate command and emit structured JSON result.
from __future__ import annotations

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from .....core.process import run_command

ROOT = Path(__file__).resolve().parents[6]


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: run_gate.py <gate_name> <command...>", file=sys.stderr)
        return 2

    gate_name = sys.argv[1]
    cmd = sys.argv[2:]
    run_id = os.environ.get("RUN_ID", datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
    proc = run_command(cmd, cwd=ROOT)

    payload = {
        "gate": gate_name,
        "run_id": run_id,
        "status": "pass" if proc.code == 0 else "fail",
        "exit_code": proc.code,
        "command": cmd,
        "artifacts": {},
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
    }

    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    print(f"gate-result: {json.dumps(payload, sort_keys=True)}")

    return proc.code


if __name__ == "__main__":
    raise SystemExit(main())
