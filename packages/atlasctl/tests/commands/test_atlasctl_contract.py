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


def _atlasctl_schema() -> dict[str, object]:
    return json.loads((ROOT / "configs/contracts/atlasctl-output.schema.json").read_text(encoding="utf-8"))


def test_version_json_matches_atlasctl_schema() -> None:
    proc = _run_cli("--json", "version")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    jsonschema.validate(payload, _atlasctl_schema())


def test_self_check_json_matches_atlasctl_schema() -> None:
    proc = _run_cli("--json", "self-check")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    jsonschema.validate(payload, _atlasctl_schema())


def test_explain_json_matches_atlasctl_schema() -> None:
    proc = _run_cli("--json", "explain", "check")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    jsonschema.validate(payload, _atlasctl_schema())
    assert payload["command"] == "check"
