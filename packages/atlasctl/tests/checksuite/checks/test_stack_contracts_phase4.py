from __future__ import annotations

import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_stack_versions_ssot_split_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_stack_versions_ssot_split.py")
    assert proc.returncode == 0, proc.stderr


def test_stack_profiles_match_install_matrix_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_stack_profiles_match_install_matrix.py")
    assert proc.returncode == 0, proc.stderr


def test_stack_health_report_sample_matches_schema() -> None:
    schema = json.loads((ROOT / "ops/_schemas/report/stack-health-report.schema.json").read_text(encoding="utf-8"))
    sample = json.loads((ROOT / "ops/stack/tests/goldens/stack-health-report.sample.json").read_text(encoding="utf-8"))
    Draft202012Validator(schema).validate(sample)


def test_stack_ports_inventory_sample_matches_schema() -> None:
    schema = json.loads((ROOT / "ops/_schemas/report/stack-ports-inventory.schema.json").read_text(encoding="utf-8"))
    sample = json.loads((ROOT / "ops/stack/tests/goldens/stack-ports-inventory.sample.json").read_text(encoding="utf-8"))
    Draft202012Validator(schema).validate(sample)

