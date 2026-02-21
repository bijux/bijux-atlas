from __future__ import annotations

import json
from pathlib import Path

from atlasctl.contracts.validate import validate
from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[1]


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


def _normalized_payload(text: str) -> dict[str, object]:
    payload = json.loads(text)
    if "run_id" in payload:
        payload["run_id"] = ""
    return payload


def test_help_json_golden() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    validate("atlasctl.help.v1", json.loads(proc.stdout))
    assert _normalized_payload(proc.stdout) == _normalized_payload(_golden("help.json.golden"))


def test_commands_json_golden() -> None:
    proc = run_atlasctl("--quiet", "commands", "--json")
    assert proc.returncode == 0, proc.stderr
    validate("atlasctl.commands.v1", json.loads(proc.stdout))
    assert _normalized_payload(proc.stdout) == _normalized_payload(_golden("commands.json.golden"))


def test_surface_json_golden() -> None:
    proc = run_atlasctl("--quiet", "surface", "--json")
    assert proc.returncode == 0, proc.stderr
    validate("atlasctl.surface.v1", json.loads(proc.stdout))
    assert _normalized_payload(proc.stdout) == _normalized_payload(_golden("surface.json.golden"))


def test_explain_json_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "explain", "check")
    assert proc.returncode == 0, proc.stderr
    validate("atlasctl.explain.v1", json.loads(proc.stdout))
    assert _normalized_payload(proc.stdout) == _normalized_payload(_golden("explain.check.json.golden"))


def test_commands_out_file_is_validated() -> None:
    proc = run_atlasctl("commands", "--json", "--out-file", "ops/_evidence/forbidden.json")
    assert proc.returncode == 3
    assert "forbidden write path" in proc.stderr
