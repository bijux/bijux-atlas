from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "tools/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


@pytest.mark.unit
def test_commands_json_matches_surface_schema() -> None:
    proc = _run_cli("commands", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-surface-output.schema.json").read_text(encoding="utf-8"))
    import jsonschema

    jsonschema.validate(payload, schema)


@pytest.mark.unit
def test_commands_output_is_deterministic() -> None:
    first = _run_cli("commands", "--json")
    second = _run_cli("commands", "--json")
    assert first.returncode == 0, first.stderr
    assert second.returncode == 0, second.stderr
    assert first.stdout == second.stdout


@pytest.mark.unit
def test_command_surface_documented_in_tooling_page() -> None:
    proc = _run_cli("commands", "--json")
    assert proc.returncode == 0, proc.stderr
    commands = [entry["command"] for entry in json.loads(proc.stdout)["commands"]]
    for command in (
        "bijux-atlas-scripts doctor",
        "bijux-atlas-scripts ops",
        "bijux-atlas-scripts make",
        "bijux-atlas-scripts report",
    ):
        assert command in commands
