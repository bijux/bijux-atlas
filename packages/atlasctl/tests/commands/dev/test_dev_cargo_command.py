from __future__ import annotations

import argparse
import json

from atlasctl.commands.dev.command import run_dev_command
from atlasctl.commands.dev.cargo.command import DevCargoParams, run_dev_cargo
from atlasctl.core.context import RunContext
from atlasctl.core.errors import ScriptError
import pytest


def test_dev_fmt_invokes_cargo_runtime(monkeypatch) -> None:
    seen: dict[str, object] = {}

    def fake_run(_ctx: RunContext, params: DevCargoParams) -> int:
        seen["action"] = params.action
        seen["json"] = params.json_output
        return 0

    monkeypatch.setattr("atlasctl.commands.dev.command.run_dev_cargo", fake_run)
    ctx = RunContext.from_args("dev-fmt", None, "test", False)
    ns = argparse.Namespace(dev_cmd="fmt", json=True, verbose=False)
    rc = run_dev_command(ctx, ns)
    assert rc == 0
    assert seen == {"action": "fmt", "json": True}


def test_dev_test_flags_map_to_runtime(monkeypatch) -> None:
    seen: dict[str, object] = {}

    def fake_run(_ctx: RunContext, params: DevCargoParams) -> int:
        seen["all"] = params.all_tests
        seen["contracts"] = params.contracts_tests
        return 0

    monkeypatch.setattr("atlasctl.commands.dev.command.run_dev_cargo", fake_run)
    ctx = RunContext.from_args("dev-test", None, "test", False)
    ns = argparse.Namespace(dev_cmd="test", all=True, contracts=False, args=[], json=False, verbose=False)
    rc = run_dev_command(ctx, ns)
    assert rc == 0
    assert seen == {"all": True, "contracts": False}


def test_dev_test_with_args_forwards_to_test(monkeypatch) -> None:
    seen: list[str] = []

    def fake_forward(_ctx: RunContext, *args: str) -> int:
        seen.extend(args)
        return 0

    monkeypatch.setattr("atlasctl.commands.dev.command._forward", fake_forward)
    ctx = RunContext.from_args("dev-test-args", None, "test", False)
    ns = argparse.Namespace(dev_cmd="test", all=False, contracts=False, args=["run", "integration"], json=False, verbose=False)
    rc = run_dev_command(ctx, ns)
    assert rc == 0
    assert seen == ["test", "run", "integration"]


def test_run_dev_cargo_json_payload(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_subprocess_run(cmd, **_kwargs):
        calls.append(cmd)
        class P:
            returncode = 0
            stdout = ""
            stderr = ""
        return P()

    monkeypatch.setattr("atlasctl.core.effects.dev_cargo.subprocess.run", fake_subprocess_run)
    ctx = RunContext.from_args("dev-json", None, "test", False)
    rc = run_dev_cargo(ctx, DevCargoParams(action="check", json_output=True, verbose=False))
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    assert payload["schema_name"] == "atlasctl.output-base.v2"
    assert payload["ok"] is True
    assert calls


def test_dev_forward_propagates_quiet_and_json(monkeypatch) -> None:
    seen: list[str] = []

    def fake_subprocess_run(cmd, **_kwargs):
        seen.extend(cmd)
        class P:
            returncode = 0
        return P()

    ctx = RunContext.from_args("dev-forward-flags", None, "test", False, "json", quiet=True)
    monkeypatch.setattr("atlasctl.commands.dev.command.subprocess.run", fake_subprocess_run)
    rc = run_dev_command(ctx, argparse.Namespace(dev_cmd="ci", args=["run", "--explain"]))
    assert rc == 0
    assert "--quiet" in seen
    assert "--format" in seen and "json" in seen


def test_dev_make_help_forwards_to_make_help(monkeypatch) -> None:
    seen: list[str] = []

    def fake_subprocess_run(cmd, **_kwargs):
        seen.extend(cmd)
        class P:
            returncode = 0
        return P()

    ctx = RunContext.from_args("dev-make-help", None, "test", False)
    monkeypatch.setattr("atlasctl.commands.dev.command.subprocess.run", fake_subprocess_run)
    rc = run_dev_command(ctx, argparse.Namespace(dev_cmd="make", args=["help"]))
    assert rc == 0
    assert seen[-2:] == ["make", "help"]


def test_dev_fmt_refuses_without_isolate(monkeypatch) -> None:
    monkeypatch.setattr("atlasctl.core.effects.dev_cargo._build_isolate_env", lambda _ctx, _action: {})
    ctx = RunContext.from_args("dev-no-isolate", None, "test", False)
    with pytest.raises(ScriptError):
        run_dev_cargo(ctx, DevCargoParams(action="fmt", json_output=True, verbose=False))


def test_dev_writes_evidence_and_meta(monkeypatch, tmp_path) -> None:
    def fake_subprocess_run(_cmd, **_kwargs):
        class P:
            returncode = 0
            stdout = ""
            stderr = ""

        return P()

    monkeypatch.setattr("atlasctl.core.context.find_repo_root", lambda: tmp_path)
    monkeypatch.setattr("atlasctl.core.effects.dev_cargo.subprocess.run", fake_subprocess_run)
    ctx = RunContext.from_args("dev-evidence", None, "test", False)
    rc = run_dev_cargo(ctx, DevCargoParams(action="check", json_output=True, verbose=False))
    assert rc == 0
    out_dir = tmp_path / "artifacts" / "evidence" / "dev" / "dev-evidence"
    assert (out_dir / "run.meta.json").exists()
    assert (out_dir / "dev-check.report.json").exists()
