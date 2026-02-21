from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

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


@pytest.mark.unit
def test_help_json_includes_required_namespaces() -> None:
    proc = _run_cli("help", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = {row["name"] for row in payload["commands"]}
    required = {
        "doctor",
        "inventory",
        "gates",
        "ops",
        "docs",
        "configs",
        "policies",
        "k8s",
        "stack",
        "load",
        "obs",
        "report",
    }
    assert required.issubset(names)


@pytest.mark.unit
def test_generated_cli_doc_covers_help_json_names() -> None:
    proc = _run_cli("help", "--json")
    assert proc.returncode == 0, proc.stderr
    names = {row["name"] for row in json.loads(proc.stdout)["commands"]}
    doc = (ROOT / "docs/_generated/cli.md").read_text(encoding="utf-8")
    missing = [name for name in sorted(names) if f"- {name}" not in doc]
    assert not missing, f"docs/_generated/cli.md missing commands: {missing}"
