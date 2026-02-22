from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_doctor_output_validates_contract_schema() -> None:
    proc = _run_cli("--quiet", "doctor", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-doctor-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def test_surface_output_validates_contract_schema() -> None:
    proc = _run_cli("--quiet", "surface", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-surface-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
