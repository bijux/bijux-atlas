from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import jsonschema
import pytest

ROOT = Path(__file__).resolve().parents[3]


@pytest.mark.integration
def test_doctor_json_output_matches_schema() -> None:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "--run-id", "integration", "doctor", "--json"],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-doctor-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    assert payload["run_id"] == "integration"
