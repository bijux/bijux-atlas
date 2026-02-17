#!/usr/bin/env python3
# Purpose: helpers for deterministic script artifact directories.
# Inputs: script name and optional run id.
# Outputs: created output directory under artifacts/scripts/<name>/<run-id>/.
from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

from scripts.tools.path_utils import repo_path


def utc_run_id() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def script_output_dir(script_name: str, run_id: str | None = None) -> Path:
    resolved = run_id or utc_run_id()
    out = repo_path("artifacts", "scripts", script_name, resolved)
    out.mkdir(parents=True, exist_ok=True)
    return out
