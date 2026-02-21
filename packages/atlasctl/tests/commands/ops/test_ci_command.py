from __future__ import annotations

import argparse
import json
from types import SimpleNamespace

from atlasctl.commands.dev.ci.command import run_ci_command
from atlasctl.core.context import RunContext


def test_ci_run_invokes_suite_ci(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"kind":"suite-run","tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0}}\n', stderr="")

    monkeypatch.setattr("atlasctl.commands.dev.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-run-test", None, "test", False)
    ns = argparse.Namespace(ci_cmd="run", json=True, out_dir=None, lane=["docs"], fail_fast=True, keep_going=False, no_isolate=False, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    assert calls and any("suite" in cmd and "run" in cmd and "ci" in cmd for cmd in calls)
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["suite_result"]["kind"] == "suite-run"
    assert payload["lane_filter"] == ["docs"]
    assert payload["execution"] == "fail-fast"
    assert "artifacts" in payload


def test_ci_dependency_lock_refresh_json(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd if isinstance(cmd, list) else [str(cmd)])
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr("atlasctl.commands.dev.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-lock-test", None, "test", False)
    ns = argparse.Namespace(ci_cmd="dependency-lock-refresh", json=True, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    assert len(calls) >= 3
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["schema_name"] == "atlasctl.output-base.v2"
    assert payload["meta"]["action"] == "dependency-lock-refresh"


def test_ci_run_no_isolate_mode(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0},"results":[]}\n', stderr="")

    monkeypatch.setattr("atlasctl.commands.dev.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-run-debug", None, "test", False)
    ns = argparse.Namespace(ci_cmd="run", json=True, out_dir=None, lane=[], fail_fast=False, keep_going=True, no_isolate=True, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    assert calls
    assert calls[0][0] != "env"
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["mode"] == "debug-no-isolate"


def test_ci_run_explain_does_not_execute(monkeypatch, capsys) -> None:
    calls: list[list[str] | str] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0},"results":[]}\n', stderr="")

    monkeypatch.setattr("atlasctl.commands.dev.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-run-explain", None, "test", False)
    ns = argparse.Namespace(
        ci_cmd="run",
        json=True,
        out_dir=None,
        lane=["docs"],
        fail_fast=False,
        keep_going=True,
        no_isolate=False,
        verbose=False,
        explain=True,
    )
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    assert not any(
        ("suite" in cmd and "run" in cmd and "ci" in cmd)
        for cmd in calls
        if isinstance(cmd, list)
    )
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["action"] == "ci-run-explain"
    assert payload["planned_steps"]


def test_ci_run_runtime_invalid_json_returns_one(monkeypatch, capsys) -> None:
    def fake_run(_cmd, **_kwargs):
        return SimpleNamespace(returncode=9, stdout="not-json", stderr="boom")

    monkeypatch.setattr("atlasctl.commands.dev.ci.command.subprocess.run", fake_run)
    ctx = RunContext.from_args("ci-run-invalid-json", None, "test", False)
    ns = argparse.Namespace(
        ci_cmd="run",
        json=True,
        out_dir=None,
        lane=[],
        fail_fast=False,
        keep_going=True,
        no_isolate=True,
        verbose=False,
        explain=False,
    )
    rc = run_ci_command(ctx, ns)
    assert rc == 1
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["status"] == "error"
    assert "next" in payload and payload["next"]
