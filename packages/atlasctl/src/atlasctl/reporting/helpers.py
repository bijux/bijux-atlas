from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

from ..core.runtime.paths import find_repo_root


def utc_run_id() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def script_output_dir(script_name: str, run_id: str | None = None) -> Path:
    resolved_run_id = run_id or utc_run_id()
    out = find_repo_root() / "artifacts" / "scripts" / script_name / resolved_run_id
    out.mkdir(parents=True, exist_ok=True)
    return out

