from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from tests.helpers import golden_text

ROOT = Path(__file__).resolve().parents[4]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_check_make_uses_lint_payload() -> None:
    proc = _run_cli("check", "make")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["suite"] == "makefiles"
    assert "checks" in payload


def test_check_shell_group_runs() -> None:
    proc = _run_cli("check", "shell")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["group"] == "shell"
    assert payload["tool"] == "atlasctl"


def test_checks_rename_report_lists_legacy_aliases() -> None:
    proc = _run_cli("checks", "rename-report", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "check-rename-report"
    by_old = {row["old"]: row["new"] for row in payload["renames"]}
    assert by_old["repo.argparse_policy"] == "checks_repo_cli_argparse_policy"


def test_legacy_check_id_resolves_in_explain() -> None:
    proc = _run_cli("check", "explain", "repo.argparse_policy")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["id"] == "checks_repo_cli_argparse_policy"


def test_checks_owners_json_golden() -> None:
    proc = _run_cli("checks", "owners", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == golden_text("check/checks-owners.json.golden")
