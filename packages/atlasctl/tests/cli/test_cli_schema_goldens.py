from __future__ import annotations

import json
from pathlib import Path

from atlasctl.contracts.validate import validate
from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[1]


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


def test_surface_json_matches_schema_and_golden() -> None:
    proc = run_atlasctl("--quiet", "surface", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.surface.v1", payload)
    assert proc.stdout.strip() == _golden("surface.json.golden")


def test_check_list_json_matches_schema_and_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "check", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.check-list.v1", payload)
    assert proc.stdout.strip() == _golden("check-list.json.golden")
