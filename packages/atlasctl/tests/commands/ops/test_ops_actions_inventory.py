from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

# schema-validate-exempt: golden shape is custom list inventory, covered by dedicated assertions below.


ROOT = Path(__file__).resolve().parents[5]
GOLDEN = ROOT / "packages/atlasctl/tests/goldens/check/ops-actions-inventory.json.golden"


def test_ops_list_actions_golden() -> None:
    proc = subprocess.run(
        ["python3", "-m", "atlasctl.cli", "ops", "--list-actions", "--json"],
        cwd=ROOT / "packages" / "atlasctl",
        env={**os.environ, "PYTHONPATH": "src"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "ops-actions-inventory"
    assert GOLDEN.read_text(encoding="utf-8") == json.dumps(payload, indent=2, sort_keys=True) + "\n"
