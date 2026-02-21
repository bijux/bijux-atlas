from __future__ import annotations

from pathlib import Path

from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parent


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


def test_help_json_golden() -> None:
    proc = run_atlasctl("--quiet", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("help.json.golden")


def test_commands_json_golden() -> None:
    proc = run_atlasctl("--quiet", "commands", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("commands.json.golden")


def test_surface_json_golden() -> None:
    proc = run_atlasctl("--quiet", "surface", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("surface.json.golden")


def test_explain_json_golden() -> None:
    proc = run_atlasctl("--quiet", "--json", "explain", "check")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("explain.check.json.golden")


def test_commands_out_file_is_validated() -> None:
    proc = run_atlasctl("commands", "--json", "--out-file", "ops/_evidence/forbidden.json")
    assert proc.returncode == 3
    assert "forbidden write path" in proc.stderr
