from __future__ import annotations

import argparse
from types import SimpleNamespace

from atlasctl.ci.command import run_ci_command
from atlasctl.core.context import RunContext


def test_ci_run_invokes_suite_ci(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"kind":"suite-run","tool":"atlasctl"}\n', stderr="")

    monkeypatch.setattr("atlasctl.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-run-test", None, "test", False)
    ns = argparse.Namespace(ci_cmd="run", json=True)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    assert calls and any(cmd[-2:] == ["run", "ci"] for cmd in calls)
    out = capsys.readouterr().out.strip()
    assert '"kind":"suite-run"' in out
