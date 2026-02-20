from __future__ import annotations

import argparse
import json
from pathlib import Path

import jsonschema

from bijux_atlas_scripts.core.context import RunContext
from bijux_atlas_scripts.make.command import CHECKS, run_contracts_check
from bijux_atlas_scripts.make.target_graph import parse_make_targets, render_tree

ROOT = Path(__file__).resolve().parents[3]


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
    from bijux_atlas_scripts.make.command import configure_make_parser

    configure_make_parser(sub)
    ns = parser.parse_args(["make", "surface"])
    assert ns.cmd == "make"
    assert ns.make_cmd == "surface"
