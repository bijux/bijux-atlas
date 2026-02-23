from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_ops_load_phase4_contract_checks_pass() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_load_contracts_phase4.py")
    assert proc.returncode == 0, proc.stderr


def test_ops_load_report_determinism_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_load_reports_deterministic.py")
    assert proc.returncode == 0, proc.stderr

