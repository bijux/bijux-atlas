#!/usr/bin/env python3
# owner: platform
# purpose: shared repository path helpers for scripts.
# stability: internal
# called-by: scripts/generate_scripts_readme.py
# Purpose: expose deterministic path helpers for scripts.
# Inputs: script name and run id where applicable.
# Outputs: pathlib.Path objects under repo/artifacts.
from __future__ import annotations

from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def artifacts_scripts_dir(script_name: str, run_id: str) -> Path:
    return repo_root() / "artifacts" / "scripts" / script_name / run_id
