from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import jsonschema

from bijux_atlas_scripts.core.context import RunContext
from bijux_atlas_scripts.ops.command import LINT_CHECKS, _run_checks

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *extra, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_ops_lint_json_schema_passes(capsys) -> None:
    def ok_runner(_cmd: list[str], _repo_root: Path) -> tuple[int, str]:
        return 0, ""

    ctx = RunContext.from_args("ops-test-run", None, "test", False)
    rc = _run_checks(
        ctx,
        checks=LINT_CHECKS,
        fail_fast=False,
        report_format="json",
        emit_artifacts=False,
        runner=ok_runner,
    )
    assert rc == 0
    payload = json.loads(capsys.readouterr().out)
    schema_path = ROOT / "configs/contracts/ops-lint-output.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    assert payload["status"] == "pass"
    assert payload["total_count"] == len(LINT_CHECKS)


def test_ops_lint_fail_fast_stops_after_first_failure(capsys) -> None:
    calls: list[list[str]] = []

    def bad_runner(cmd: list[str], _repo_root: Path) -> tuple[int, str]:
        calls.append(cmd)
        return 1, "failing check"

    ctx = RunContext.from_args("ops-test-run", None, "test", False)
    rc = _run_checks(
        ctx,
        checks=LINT_CHECKS,
        fail_fast=True,
        report_format="text",
        emit_artifacts=False,
        runner=bad_runner,
    )
    assert rc == 1
    out = capsys.readouterr().out
    assert "ops lint:" in out
    assert "FAIL" in out
    assert len(calls) == 1


def test_ops_surface_integration_json_report() -> None:
    proc = _run_cli("ops", "surface", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["status"] in {"pass", "fail"}


def test_ops_contracts_check_integration() -> None:
    proc = _run_cli("ops", "contracts-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"


def test_ops_layer_drift_check_integration() -> None:
    proc = _run_cli("ops", "layer-drift-check", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"


def test_ops_suites_check_integration() -> None:
    proc = _run_cli("ops", "suites-check", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"


def test_ops_policy_audit_integration() -> None:
    proc = _run_cli("ops", "policy-audit", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["status"] in {"pass", "fail"}


def test_ops_k8s_test_contract_integration() -> None:
    proc = _run_cli("ops", "k8s-test-contract", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
