#!/usr/bin/env python3
# owner: platform
# purpose: shared repository path helpers for scripts.
# stability: internal
# called-by: atlasctl inventory/scripts migration generators
# Purpose: expose deterministic path helpers for scripts.
# Inputs: script name and run id where applicable.
# Outputs: pathlib.Path objects under repo/artifacts.
from __future__ import annotations

from pathlib import Path

from bijux_atlas_scripts.paths import artifacts_scripts_dir as _artifacts_scripts_dir
from bijux_atlas_scripts.paths import find_root as _repo_root


def repo_root() -> Path:
    return _repo_root()


def artifacts_scripts_dir(script_name: str, run_id: str) -> Path:
    return _artifacts_scripts_dir(script_name=script_name, run_id=run_id)
