from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

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


def test_help_json_command_names_match_golden() -> None:
    proc = _run_cli("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    names = [entry["name"] for entry in payload["commands"]]
    golden = (Path(__file__).resolve().parent / "goldens" / "cli_help_commands.expected.txt").read_text(
        encoding="utf-8"
    )
    expected = [line.strip() for line in golden.splitlines() if line.strip()]
    assert names == expected
