from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([sys.executable, rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_ops_product_makefile_token_purity_check_passes() -> None:
    # Run via atlasctl check to ensure registry wiring works.
    proc = subprocess.run([
        "./bin/atlasctl",
        "check",
        "run",
        "--id",
        "checks_make_ops_product_no_tool_tokens",
        "--quiet",
    ], cwd=ROOT, text=True, capture_output=True, check=False)
    assert proc.returncode == 0, proc.stdout + proc.stderr


def test_ops_product_makefile_delegation_check_passes() -> None:
    proc = subprocess.run([
        "./bin/atlasctl",
        "check",
        "run",
        "--id",
        "checks_make_ops_product_atlasctl_only_delegation",
        "--quiet",
    ], cwd=ROOT, text=True, capture_output=True, check=False)
    assert proc.returncode == 0, proc.stdout + proc.stderr
