from __future__ import annotations

from pathlib import Path

from bijux_atlas_scripts.core.git import read_git_context
from bijux_atlas_scripts.core.process import run_command
from bijux_atlas_scripts.core.tooling import read_pins, read_tool_versions

ROOT = Path(__file__).resolve().parents[3]


def test_run_command_captures_output_and_duration() -> None:
    res = run_command(["python3", "-c", "print('ok')"], ROOT)
    assert res.code == 0
    assert res.stdout.strip() == "ok"
    assert res.duration_ms >= 0


def test_git_context_present() -> None:
    ctx = read_git_context(ROOT)
    assert ctx.sha
    assert isinstance(ctx.is_dirty, bool)


def test_tooling_and_pins_load() -> None:
    tool_versions = read_tool_versions(ROOT)
    pins = read_pins(ROOT)
    assert "python3" in tool_versions
    assert isinstance(pins, dict)
