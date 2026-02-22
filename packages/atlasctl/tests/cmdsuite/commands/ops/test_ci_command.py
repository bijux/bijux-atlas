from __future__ import annotations

import argparse
import json
from types import SimpleNamespace
from pathlib import Path

import jsonschema
from atlasctl.commands.dev.ci.command import run_ci_command
from atlasctl.core.effects.dev_ci import _ci_out_dir
from atlasctl.contracts.validate import validate
from atlasctl.core.context import RunContext
from tests.helpers import ROOT, golden_path, run_atlasctl


def test_ci_run_invokes_suite_ci(monkeypatch, capsys) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"kind":"suite-run","tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0}}\n', stderr="")

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
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

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
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

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
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

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
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

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
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


def test_ci_run_writes_expected_artifacts(monkeypatch, tmp_path: Path, capsys) -> None:
    def fake_run(_cmd, **_kwargs):
        return SimpleNamespace(
            returncode=0,
            stdout='{"tool":"atlasctl","status":"ok","summary":{"passed":2,"failed":0,"skipped":0},"results":[{"label":"check repo","status":"pass"}]}\n',
            stderr="",
        )

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
    out_dir = tmp_path / "ci-out"
    ctx = RunContext.from_args("ci-run-artifacts", None, "test", False)
    ns = argparse.Namespace(
        ci_cmd="run",
        json=True,
        out_dir=str(out_dir),
        lane=[],
        fail_fast=False,
        keep_going=True,
        no_isolate=True,
        verbose=False,
        explain=False,
    )
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out.strip())
    assert (out_dir / "suite-ci.report.json").exists()
    assert (out_dir / "suite-ci.summary.txt").exists()
    assert (out_dir / "run.meta.json").exists()
    assert payload["artifacts"]["json"].endswith("suite-ci.report.json")
    assert payload["artifacts"]["meta"].endswith("run.meta.json")


def test_ci_run_lane_filter_builds_only_patterns(monkeypatch) -> None:
    calls: list[list[str]] = []

    def fake_run(cmd, **_kwargs):
        if isinstance(cmd, list):
            calls.append(cmd)
        return SimpleNamespace(returncode=0, stdout='{"tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0},"results":[]}\n', stderr="")

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
    ctx = RunContext.from_args("ci-run-lanes", None, "test", False)
    ns = argparse.Namespace(
        ci_cmd="run",
        json=True,
        out_dir=None,
        lane=["rust", "docs"],
        fail_fast=False,
        keep_going=True,
        no_isolate=True,
        verbose=False,
        explain=False,
    )
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    suite_cmd = next(cmd for cmd in calls if "suite" in cmd and "run" in cmd and "ci" in cmd)
    assert "--only" in suite_cmd
    assert "check repo*" in suite_cmd
    assert "cmd *cargo*" in suite_cmd
    assert "check docs*" in suite_cmd


def test_ci_dependency_lock_refresh_json_schema(monkeypatch, capsys) -> None:
    def fake_run(_cmd, **_kwargs):
        return SimpleNamespace(returncode=0, stdout="", stderr="")

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
    ctx = RunContext.from_args("ci-lock-schema", None, "test", False)
    ns = argparse.Namespace(ci_cmd="dependency-lock-refresh", json=True, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out.strip())
    validate("atlasctl.output-base.v2", payload)


def test_ci_out_dir_resolution(tmp_path: Path) -> None:
    ctx = RunContext.from_args("ci-out-dir", str(tmp_path / "evidence"), "test", False)
    rel = _ci_out_dir(ctx, "artifacts/evidence/ci/custom")
    abs_path = _ci_out_dir(ctx, str(tmp_path / "abs"))
    assert str(rel).endswith("artifacts/evidence/ci/custom")
    assert abs_path == tmp_path / "abs"


def test_ci_report_latest(monkeypatch, tmp_path: Path, capsys) -> None:
    old = tmp_path / "artifacts" / "evidence" / "ci" / "run-old"
    new = tmp_path / "artifacts" / "evidence" / "ci" / "run-new"
    old.mkdir(parents=True, exist_ok=True)
    new.mkdir(parents=True, exist_ok=True)
    (new / "suite-ci.report.json").write_text("{}\n", encoding="utf-8")
    (new / "suite-ci.summary.txt").write_text("ok\n", encoding="utf-8")
    (new / "run.meta.json").write_text("{}\n", encoding="utf-8")
    monkeypatch.setattr("atlasctl.core.context.find_repo_root", lambda: tmp_path)

    ctx = RunContext.from_args("ci-report", None, "test", False)
    ns = argparse.Namespace(ci_cmd="report", latest=True, json=True, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["run_id"] == "run-new"
    assert payload["artifacts"]["meta"].endswith("run.meta.json")


def test_ci_report_requires_latest_flag(capsys) -> None:
    ctx = RunContext.from_args("ci-report-arg", None, "test", False)
    ns = argparse.Namespace(ci_cmd="report", latest=False, json=True, verbose=False)
    rc = run_ci_command(ctx, ns)
    assert rc == 2
    payload = json.loads(capsys.readouterr().out.strip())
    assert payload["status"] == "error"


def test_ci_list_json_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "ci", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    golden = json.loads(golden_path("list/ci-list.json.golden").read_text(encoding="utf-8"))
    assert payload == golden


def test_ci_run_json_schema_contract(monkeypatch, capsys) -> None:
    def fake_run(_cmd, **_kwargs):
        return SimpleNamespace(
            returncode=0,
            stdout='{"tool":"atlasctl","status":"ok","summary":{"passed":1,"failed":0,"skipped":0},"results":[{"label":"check docs","status":"pass"}]}\n',
            stderr="",
        )

    monkeypatch.setattr("atlasctl.core.effects.dev_ci.process_run", fake_run)
    ctx = RunContext.from_args("ci-run-schema-contract", None, "test", False)
    ns = argparse.Namespace(
        ci_cmd="run",
        json=True,
        out_dir=None,
        lane=["docs"],
        fail_fast=False,
        keep_going=True,
        no_isolate=True,
        verbose=False,
        explain=False,
    )
    rc = run_ci_command(ctx, ns)
    assert rc == 0
    payload = json.loads(capsys.readouterr().out.strip())
    schema = json.loads((ROOT / "configs/contracts/ci-run-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
