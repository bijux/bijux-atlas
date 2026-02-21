from __future__ import annotations

import json
from pathlib import Path

from atlasctl.contracts.validate import validate
from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[1]


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


def test_suite_refgrade_list_output_golden() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "refgrade", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("suite_refgrade.expected.txt")


def test_suite_run_json_schema_contract(tmp_path) -> None:
    target = tmp_path / "suite-run"
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check repo.module_size", "--json", "--target-dir", str(target))
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_name"] == "atlasctl.suite-run.v1"
    validate("atlasctl.suite-run.v1", payload)
