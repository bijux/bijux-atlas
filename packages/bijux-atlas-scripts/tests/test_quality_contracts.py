from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]


def _cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )


@pytest.mark.unit
def test_inventory_build_is_deterministic() -> None:
    first = _cli("inventory", "make", "--format", "json", "--dry-run")
    second = _cli("inventory", "make", "--format", "json", "--dry-run")
    assert first.returncode == 0, first.stderr
    assert second.returncode == 0, second.stderr
    assert first.stdout == second.stdout


@pytest.mark.unit
def test_validate_output_failure_is_human_readable(tmp_path: Path) -> None:
    bad = tmp_path / "bad.json"
    bad.write_text('{"oops": 1}\n', encoding="utf-8")
    proc = _cli(
        "validate-output",
        "--schema",
        "configs/contracts/scripts-doctor-output.schema.json",
        "--file",
        str(bad),
    )
    assert proc.returncode != 0
    assert "validation error" in proc.stderr.lower() or "failed" in proc.stderr.lower()


@pytest.mark.integration
def test_doctor_includes_pins_and_tool_versions() -> None:
    proc = _cli("--run-id", "doctor-quality", "doctor", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert "pins" in payload
    assert "tool_versions" in payload["pins"]
