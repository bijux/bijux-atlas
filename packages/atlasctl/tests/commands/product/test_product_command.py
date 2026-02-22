from __future__ import annotations

import argparse

from atlasctl.commands.product.command import run_product_command
from atlasctl.core.context import RunContext


def _ctx() -> RunContext:
    return RunContext.from_args("product-cmd-test", None, "test", False)


def test_product_command_dispatches_migrated_lanes(monkeypatch) -> None:
    calls: list[tuple[str, int]] = []

    def fake_run_product_lane(ctx, *, lane, steps, meta=None):  # type: ignore[no-untyped-def]
        calls.append((lane, len(steps)))
        return 0

    monkeypatch.setattr("atlasctl.commands.product.command.run_product_lane", fake_run_product_lane)

    cases = [
        argparse.Namespace(product_cmd="bootstrap"),
        argparse.Namespace(product_cmd="docker", product_docker_cmd="build"),
        argparse.Namespace(product_cmd="docker", product_docker_cmd="check"),
        argparse.Namespace(product_cmd="docker", product_docker_cmd="contracts"),
        argparse.Namespace(product_cmd="chart", product_chart_cmd="package"),
        argparse.Namespace(product_cmd="chart", product_chart_cmd="verify"),
        argparse.Namespace(product_cmd="chart", product_chart_cmd="validate"),
        argparse.Namespace(product_cmd="naming", product_naming_cmd="lint"),
        argparse.Namespace(product_cmd="docs", product_docs_cmd="naming-lint"),
        argparse.Namespace(product_cmd="check"),
    ]
    for ns in cases:
        assert run_product_command(_ctx(), ns) == 0

    assert [name for name, _ in calls] == [
        "bootstrap",
        "docker build",
        "docker check",
        "docker contracts",
        "chart package",
        "chart verify",
        "chart validate",
        "naming lint",
        "docs naming-lint",
        "check",
    ]


def test_product_docker_push_requires_ci(monkeypatch, capsys) -> None:
    monkeypatch.delenv("CI", raising=False)
    monkeypatch.setenv("DOCKER_IMAGE", "x:y")
    ns = argparse.Namespace(product_cmd="docker", product_docker_cmd="push")
    assert run_product_command(_ctx(), ns) == 2
    assert "CI-only" in capsys.readouterr().out

