from __future__ import annotations

import argparse

from atlasctl.commands.dev.command import _run_regen_registry
from atlasctl.core.context import RunContext


def test_regen_registry_runs_generator_then_diff(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []

    class _Proc:
        def __init__(self, returncode: int) -> None:
            self.returncode = returncode

    def fake_run(cmd, **_kwargs):
        calls.append(list(cmd))
        if cmd[:3] == ["git", "diff", "--exit-code"]:
            return _Proc(0)
        return _Proc(0)

    monkeypatch.setattr("atlasctl.commands.dev.command.run", fake_run)
    monkeypatch.setattr("atlasctl.core.context.find_repo_root", lambda: tmp_path)
    ctx = RunContext.from_args("dev-regen-registry", None, "test", False)
    rc = _run_regen_registry(ctx, argparse.Namespace(json=True))
    assert rc == 0
    assert calls[0][:4] == ["python", "-m", "atlasctl.cli", "gen"] or calls[0][1:4] == ["-m", "atlasctl.cli", "gen"]
    assert calls[1][:3] == ["git", "diff", "--exit-code"]

