from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_ops_import_boundary_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_command_import_boundary.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_internal_public_boundary_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_internal_not_public.py")
    assert proc.returncode == 0, proc.stderr
