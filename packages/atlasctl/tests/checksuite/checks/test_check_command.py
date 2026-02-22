from __future__ import annotations

# schema-validate-exempt: check-owners payload has no dedicated schema contract yet.
import json
import subprocess
import sys
from pathlib import Path

from tests.helpers import golden_text

ROOT = Path(__file__).resolve().parents[5]


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
    proc = _run_cli("check", "explain", "checks_repo_cli_argparse_policy")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["id"] == "checks_repo_cli_argparse_policy"


def test_checks_owners_json_golden() -> None:
    proc = _run_cli("checks", "owners", "--json")
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == golden_text("check/checks-owners.json.golden")


def test_checks_failures_json_report() -> None:
    report = ROOT / "artifacts/evidence/checks/test-check-failures.json"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "tool": "atlasctl",
                "kind": "check-run-report",
                "rows": [
                    {"id": "checks_repo_cli_argparse_policy", "domain": "repo", "status": "FAIL", "hint": "h", "detail": "d"},
                    {"id": "checks_docs_links_exist", "domain": "docs", "status": "PASS", "hint": "", "detail": ""},
                ],
            }
        ),
        encoding="utf-8",
    )
    proc = _run_cli("checks", "failures", "--last-run", str(report), "--group", "repo", "--json")
    assert proc.returncode == 2, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "check-failures"
    assert payload["failed_count"] == 1


def test_check_run_list_selected_is_sorted() -> None:
    proc = _run_cli("check", "run", "--group", "repo", "--list-selected", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "check-selection"
    assert payload["checks"] == sorted(payload["checks"])


def test_check_run_marker_and_exclude_marker_filters() -> None:
    proc = _run_cli("check", "run", "-m", "slow", "--exclude-marker", "slow", "--list-selected", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["count"] == 0


def test_check_run_exclude_group_filter() -> None:
    proc = _run_cli("check", "run", "--group", "repo", "--exclude-group", "repo", "--list-selected", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["count"] == 0


def test_checks_gates_json() -> None:
    proc = _run_cli("checks", "gates", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "check-gates"
    assert any(row["gate"] == "repo" for row in payload["gates"])
