from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_e2e_scenario_contracts_use_unified_schema_and_actions() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_e2e_scenarios.py")
    assert proc.returncode == 0, proc.stderr
    proc2 = _run("packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_realdata_scenarios.py")
    assert proc2.returncode == 0, proc2.stderr
    proc3 = _run("packages/atlasctl/src/atlasctl/checks/layout/domains/scenarios/check_e2e_suites.py")
    assert proc3.returncode == 0, proc3.stderr

