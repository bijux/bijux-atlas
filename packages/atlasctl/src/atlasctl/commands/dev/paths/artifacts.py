from __future__ import annotations

from pathlib import Path

from ....core.paths import find_repo_root


def artifacts_scripts_dir(script_name: str, run_id: str) -> Path:
    return find_repo_root() / "artifacts" / "scripts" / script_name / run_id
