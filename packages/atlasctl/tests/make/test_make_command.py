from __future__ import annotations

import argparse
import json
from types import SimpleNamespace
from pathlib import Path

import jsonschema
from atlasctl.core.context import RunContext
from atlasctl.commands.dev.make.command import run_make_command
from atlasctl.commands.dev.make.contracts_check import CHECKS, run_contracts_check
from atlasctl.commands.dev.make.dev_ci_target_map import build_dev_ci_target_payload
from atlasctl.commands.dev.make.target_graph import parse_make_targets, render_tree

ROOT = Path(__file__).resolve().parents[4]


def test_parse_make_targets_from_fixture(tmp_path: Path) -> None:
    mk = tmp_path / "mini.mk"
    mk.write_text(
        "\n".join(
            [
                "root: lint test",
                "\t$(MAKE) internal/check",
                "lint:",
                "\t@echo lint",
                "test:",
                "\t@echo test",
                "internal/check:",
                "\t@echo ok",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    graph = parse_make_targets(tmp_path)
    assert graph["root"] == ["lint", "test", "internal/check"]
    tree = render_tree(graph, "root")
    assert tree[0] == "root"
    assert any("internal/check" in line for line in tree)


def test_contracts_check_json_schema_passes(capsys) -> None:
    def ok_runner(_cmd: list[str], _repo_root: Path) -> tuple[int, str]:
        return 0, ""

    ctx = RunContext.from_args("test-run", None, "test", False)
    rc = run_contracts_check(ctx, fail_fast=False, emit_artifacts=False, as_json=True, runner=ok_runner)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    schema_path = ROOT / "configs/contracts/make-contracts-check-output.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    assert payload["status"] == "pass"
    assert payload["failed_count"] == 0
    assert payload["total_count"] == len(CHECKS)


def test_contracts_check_fail_fast_and_actionable_output(capsys) -> None:
    calls: list[list[str]] = []

    def failing_runner(cmd: list[str], _repo_root: Path) -> tuple[int, str]:
        calls.append(cmd)
        return 1, "simulated failure detail"

    ctx = RunContext.from_args("test-run", None, "test", False)
    rc = run_contracts_check(ctx, fail_fast=True, emit_artifacts=False, as_json=False, runner=failing_runner)
    assert rc == 1
    out = capsys.readouterr().out
    assert "make contracts-check: status=fail" in out
    assert "- FAIL " in out
    assert "fix: " in out
    assert len(calls) == 1


def test_make_surface_parser_shape() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    from atlasctl.commands.dev.make.command import configure_make_parser

    configure_make_parser(sub)
    ns = parser.parse_args(["make", "surface"])
    assert ns.cmd == "make"
    assert ns.make_cmd == "surface"


def test_make_parser_supports_new_subcommands() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    from atlasctl.commands.dev.make.command import configure_make_parser

    configure_make_parser(sub)
    for argv in (
        ["make", "list-targets", "--json"],
        ["make", "list-public-targets", "--json"],
        ["make", "inventory-logic", "--json"],
        ["make", "lint", "--json"],
        ["make", "rewrite", "--json"],
        ["make", "doctor", "--json"],
        ["make", "run", "ci"],
        ["make", "dev-ci-target-map", "--json"],
    ):
        ns = parser.parse_args(argv)
        assert ns.cmd == "make"


def test_make_run_rejects_non_public_target() -> None:
    ctx = RunContext.from_args("make-run-guard", None, "test", False)
    ns = argparse.Namespace(make_cmd="run", target="internal/ci/scripts-path-usage", args=[], json=True)
    rc = run_make_command(ctx, ns)
    assert rc == 2


def test_make_run_writes_evidence(monkeypatch) -> None:
    from atlasctl.commands.dev.make import command as make_command

    def fake_run(*_args, **_kwargs):
        return SimpleNamespace(returncode=0, stdout="ok\n", stderr="")

    monkeypatch.setattr(make_command.subprocess, "run", fake_run)
    ctx = RunContext.from_args("make-run-test", None, "test", False)
    ns = argparse.Namespace(make_cmd="run", target="ci", args=[], json=True)
    rc = run_make_command(ctx, ns)
    assert rc == 0
    out_path = (
        ctx.evidence_root
        / "make"
        / ctx.run_id
        / "run-ci.json"
    )
    assert out_path.exists()


def test_make_rewrite_preview_json() -> None:
    ctx = RunContext.from_args("make-rewrite-test", None, "test", False)
    ns = argparse.Namespace(make_cmd="rewrite", write=False, limit=2, json=True)
    rc = run_make_command(ctx, ns)
    assert rc == 0


def test_dev_ci_target_map_payload_is_complete_for_repo() -> None:
    payload = build_dev_ci_target_payload(ROOT)
    assert payload["status"] == "ok"
    errors = payload["errors"]
    assert errors["unmapped"] == []
    assert errors["duplicate_without_alias"] == []
    rows = payload["target_map"]
    by_target = {row["target"]: row for row in rows}
    assert by_target["ci-fmt"]["atlasctl"] == "atlasctl dev fmt"
    assert by_target["ci-test-nextest"]["atlasctl"] == "atlasctl dev test"
    assert by_target["ci-deny"]["atlasctl"] == "atlasctl dev audit"
