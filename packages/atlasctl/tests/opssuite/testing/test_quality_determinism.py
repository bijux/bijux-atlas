from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from atlasctl.checks.repo.contracts.test_guardrails import check_check_test_coverage
from atlasctl.contracts.validate import validate
from tests.helpers import ROOT, run_atlasctl


_ISO_TS_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}")


def test_test_runner_uses_strict_env_defaults() -> None:
    proc = run_atlasctl("--quiet", "test", "smoke", "--json")
    assert proc.returncode in {0, 1}, proc.stderr
    first_line = proc.stdout.splitlines()[0]
    payload = json.loads(first_line)
    assert payload["strict_env"] == {
        "PYTHONHASHSEED": "0",
        "TZ": "UTC",
        "LC_ALL": "C",
        "LANG": "C",
    }


def test_listing_output_stable_across_runs() -> None:
    a = run_atlasctl("--quiet", "list", "commands", "--json")
    b = run_atlasctl("--quiet", "list", "commands", "--json")
    assert a.returncode == 0 and b.returncode == 0
    assert json.loads(a.stdout) == json.loads(b.stdout)


def test_check_ordering_is_stable() -> None:
    first = run_atlasctl("--quiet", "--json", "check", "list")
    second = run_atlasctl("--quiet", "--json", "check", "list")
    assert first.returncode == 0 and second.returncode == 0
    first_ids = [item["id"] for item in json.loads(first.stdout)["checks"]]
    second_ids = [item["id"] for item in json.loads(second.stdout)["checks"]]
    assert first_ids == second_ids
    assert first_ids == sorted(first_ids)


def test_default_text_outputs_do_not_emit_iso_timestamps() -> None:
    for args in (
        ("--quiet", "docs", "--list"),
        ("--quiet", "suite", "--list"),
        ("--quiet", "check", "list"),
    ):
        proc = run_atlasctl(*args)
        assert proc.returncode == 0, (args, proc.stderr)
        assert _ISO_TS_RE.search(proc.stdout) is None


def test_json_outputs_validate_against_declared_schemas() -> None:
    commands = [
        ("--quiet", "help", "--json"),
        ("--quiet", "commands", "--json"),
        ("--quiet", "surface", "--json"),
        ("--quiet", "list", "checks", "--json"),
        ("--quiet", "suite", "run", "docs", "--list", "--json"),
    ]
    for args in commands:
        proc = run_atlasctl(*args)
        assert proc.returncode == 0, (args, proc.stderr)
        payload = json.loads(proc.stdout)
        schema_name = payload.get("schema_name")
        if isinstance(schema_name, str) and schema_name:
            validate(schema_name, payload)


def test_every_check_has_test_coverage_marker_or_test() -> None:
    code, errors = check_check_test_coverage(ROOT)
    assert code == 0, errors


def test_suite_coverage_is_complete() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    assert not payload.get("errors")


def test_repeated_cli_calls_are_state_isolated() -> None:
    proc1 = run_atlasctl("--quiet", "suite", "list", "--json")
    proc2 = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc1.returncode == 0 and proc2.returncode == 0
    assert json.loads(proc1.stdout) == json.loads(proc2.stdout)


def test_help_and_commands_work_outside_git_checkout() -> None:
    with tempfile.TemporaryDirectory(prefix="atlasctl-outside-git-") as td:
        env = os.environ.copy()
        env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
        cwd = Path(td)
        help_proc = subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", "help", "--json"],
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        assert help_proc.returncode == 0, help_proc.stderr
        commands_proc = subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", "commands", "--json"],
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        assert commands_proc.returncode == 0, commands_proc.stderr
