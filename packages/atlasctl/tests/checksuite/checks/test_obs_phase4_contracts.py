from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def test_obs_contract_goldens_policy_check_passes() -> None:
    proc = subprocess.run(
        ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_obs_contract_goldens_atlasctl_only.py"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr

