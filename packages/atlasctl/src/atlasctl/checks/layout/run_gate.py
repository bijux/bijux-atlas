#!/usr/bin/env python3
# Purpose: run one gate command and emit structured JSON result.
from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: run_gate.py <gate_name> <command...>", file=sys.stderr)
        return 2

    gate_name = sys.argv[1]
    cmd = sys.argv[2:]
    run_id = os.environ.get("RUN_ID", datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
    out_dir = ROOT / "ops" / "_evidence" / "gates" / run_id
    out_dir.mkdir(parents=True, exist_ok=True)

    proc = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True)
    stdout_path = out_dir / f"{gate_name}.stdout.log"
    stderr_path = out_dir / f"{gate_name}.stderr.log"
    stdout_path.write_text(proc.stdout, encoding="utf-8")
    stderr_path.write_text(proc.stderr, encoding="utf-8")

    payload = {
        "gate": gate_name,
        "run_id": run_id,
        "status": "pass" if proc.returncode == 0 else "fail",
        "exit_code": proc.returncode,
        "command": cmd,
        "artifacts": {
            "stdout": stdout_path.relative_to(ROOT).as_posix(),
            "stderr": stderr_path.relative_to(ROOT).as_posix(),
        },
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
    }
    json_path = out_dir / f"{gate_name}.json"
    json_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, file=sys.stderr, end="")
    print(f"gate-result: {json_path.relative_to(ROOT).as_posix()}")

    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
