from __future__ import annotations

import argparse

from atlasctl.commands.ops.runtime_modules.ops_runtime_parser import configure_ops_parser
from atlasctl.commands.ops.runtime_modules.ops_runtime_run import run_ops_command


class _Ctx:
    run_id = "run-1"
    repo_root = None


def _parse(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="atlasctl")
    sub = parser.add_subparsers(dest="cmd", required=True)
    configure_ops_parser(sub)
    return parser.parse_args(argv)


def test_ops_phase6_parser_adds_new_first_class_surfaces() -> None:
    ns = _parse(["ops", "platform", "up"])
    assert ns.ops_cmd == "platform"
    assert ns.ops_platform_cmd == "up"
    ns = _parse(["ops", "test", "smoke"])
    assert ns.ops_cmd == "test"
    assert ns.ops_test_cmd == "smoke"
    ns = _parse(["ops", "evidence", "collect", "--namespace", "atlas-e2e"])
    assert ns.ops_cmd == "evidence"
    assert ns.ops_evidence_cmd == "collect"


def test_ops_env_doctor_dispatch_json(monkeypatch, capsys) -> None:
    payload = {
        "schema_name": "atlasctl.ops-env-doctor.v1",
        "kind": "ops-env-doctor",
        "run_id": "run-1",
        "status": "ok",
        "toolchain_source": "ops/inventory/toolchain.yaml",
        "tools": [],
        "images": [],
    }
    monkeypatch.setattr(
        "atlasctl.commands.ops.runtime_modules.ops_runtime_run.ops_env_doctor",
        lambda ctx: payload,
    )
    code = run_ops_command(_Ctx(), argparse.Namespace(ops_cmd="env", ops_env_cmd="doctor", report="json"))
    assert code == 0
    assert "ops-env-doctor" in capsys.readouterr().out


def test_ops_platform_dispatch_routes_to_workflow(monkeypatch) -> None:
    called = {"up": False}

    def _fake(ctx, report_format: str) -> int:
        called["up"] = True
        assert report_format == "text"
        return 0

    monkeypatch.setattr(
        "atlasctl.commands.ops.runtime_modules.ops_runtime_run.ops_platform_up",
        _fake,
    )
    code = run_ops_command(_Ctx(), argparse.Namespace(ops_cmd="platform", ops_platform_cmd="up", report="text"))
    assert code == 0
    assert called["up"] is True
