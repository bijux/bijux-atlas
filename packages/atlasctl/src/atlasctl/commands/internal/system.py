from __future__ import annotations

import os
import socket
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from ...contracts.json import write_json
from ...core.repo_root import find_repo_root
from ...reporting.helpers import script_output_dir, utc_run_id


def repo_root() -> Path:
    return find_repo_root()


def dump_env(script_name: str = "env-dump", run_id: str | None = None) -> Path:
    resolved_run_id = run_id or utc_run_id()
    out_dir = script_output_dir(script_name, resolved_run_id)
    out_file = out_dir / "env.txt"
    env_lines = [f"{k}={v}" for k, v in sorted(os.environ.items())]
    payload_lines = [
        f"pwd={os.getcwd()}",
        f"repo_root={find_repo_root()}",
        f"run_id={resolved_run_id}",
        f"timestamp_utc={datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        *env_lines,
    ]
    out_file.write_text("\n".join(payload_lines) + "\n", encoding="utf-8")
    return out_file


def run_timed(cmd: list[str], script_name: str = "exec", run_id: str | None = None) -> tuple[int, Path]:
    resolved_run_id = run_id or utc_run_id()
    out_dir = script_output_dir(script_name, resolved_run_id)
    start_epoch = int(time.time())
    proc = subprocess.run(cmd, check=False)
    end_epoch = int(time.time())
    timing = {
        "script": script_name,
        "run_id": resolved_run_id,
        "started": start_epoch,
        "ended": end_epoch,
        "duration_sec": end_epoch - start_epoch,
        "exit_code": proc.returncode,
        "host": socket.gethostname(),
    }
    timing_path = write_json(out_dir / "timing.json", timing)
    return proc.returncode, timing_path
