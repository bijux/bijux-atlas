#!/usr/bin/env python3
# owner: platform
# purpose: shared repository path helpers for scripts.
# stability: internal
# called-by: scripts/areas/gen/generate_scripts_readme.py
# Purpose: expose deterministic path helpers for scripts.
# Inputs: script name and run id where applicable.
# Outputs: pathlib.Path objects under repo/artifacts.
from __future__ import annotations

from pathlib import Path

from scripts.areas.tools.path_utils import repo_root as _repo_root


def repo_root() -> Path:
    return _repo_root()


def artifacts_scripts_dir(script_name: str, run_id: str) -> Path:
    return repo_root() / "artifacts" / "scripts" / script_name / run_id
