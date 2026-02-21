from __future__ import annotations

import argparse
import json

from atlasctl.commands.dev.command import run_dev_command
from atlasctl.commands.dev.cargo.command import DevCargoParams, run_dev_cargo
from atlasctl.core.context import RunContext


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

    monkeypatch.setattr("atlasctl.commands.dev.cargo.command.subprocess.run", fake_subprocess_run)
    ctx = RunContext.from_args("dev-json", None, "test", False)
    rc = run_dev_cargo(ctx, DevCargoParams(action="check", json_output=True, verbose=False))
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    assert payload["schema_name"] == "atlasctl.output-base.v2"
    assert payload["ok"] is True
    assert calls

