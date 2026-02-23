from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def test_commands_inventory_includes_ops_and_product() -> None:
    proc = subprocess.run(
        [sys.executable, 'packages/atlasctl/src/atlasctl/checks/layout/docs/check_commands_inventory_ops_product.py'],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr
