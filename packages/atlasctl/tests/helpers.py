from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def run_atlasctl(*args: str, cwd: Path | None = None, evidence_root: Path | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
    if evidence_root is not None:
        env["EVIDENCE_ROOT"] = str(evidence_root)
        env.setdefault("RUN_ID", "pytest-run")
        env.setdefault("PROFILE", "test")
        return subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", *args],
            cwd=(cwd or ROOT),
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
    with tempfile.TemporaryDirectory(prefix="atlasctl-evidence-") as td:
        env["EVIDENCE_ROOT"] = td
        env.setdefault("RUN_ID", "pytest-run")
        env.setdefault("PROFILE", "test")
        return subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", *args],
            cwd=(cwd or ROOT),
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )


def run_atlasctl_isolated(tmp_path: Path, *args: str, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    evidence_root = tmp_path / "evidence"
    evidence_root.mkdir(parents=True, exist_ok=True)
    return run_atlasctl(*args, cwd=cwd, evidence_root=evidence_root)
