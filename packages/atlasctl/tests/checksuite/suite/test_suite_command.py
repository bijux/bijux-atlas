from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_suite_list_json() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["default"] == "required"
    names = [item["name"] for item in payload["suites"]]
    assert {"fast", "required", "ops", "ci", "required_proof"}.issubset(set(names))


def test_suite_run_list_required() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "required", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "required"
    assert payload["total_count"] >= 1
    assert payload["check_ids"]


def test_suite_run_only_single_check() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check checks_repo_module_size", "--json")
    assert proc.returncode in {0, 2}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "fast"
    assert payload["summary"]["passed"] + payload["summary"]["failed"] >= 0
    assert "results" in payload


def test_suite_inventory_check_json() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"


def test_suite_run_writes_target_dir(tmp_path) -> None:
    target = tmp_path / "suite-out"
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check checks_repo_module_size", "--json", "--target-dir", str(target))
    assert proc.returncode in {0, 2}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["target_dir"] == str(target)
    assert (target / "results.json").exists()


def test_ci_suite_stable_and_contains_expected_items() -> None:
    proc1 = run_atlasctl("--quiet", "suite", "run", "ci", "--list", "--json")
    proc2 = run_atlasctl("--quiet", "suite", "run", "ci", "--list", "--json")
    assert proc1.returncode == 0, proc1.stderr
    assert proc2.returncode == 0, proc2.stderr
    payload1 = json.loads(proc1.stdout)
    payload2 = json.loads(proc2.stdout)
    assert payload1["check_ids"] == payload2["check_ids"]
    assert "checks_repo_module_size" in set(payload1["check_ids"])


def test_required_proof_suite_contains_release_gates() -> None:
    proc = run_atlasctl("--quiet", "suite", "required_proof", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    tasks = set(payload["check_ids"])
    assert "checks_repo_dir_budget_py_files" in tasks
    assert "checks_repo_single_registry_module" in tasks
    assert "checks_repo_legacy_package_absent" in tasks
    assert "checks_repo_legacy_zero_importers" in tasks


def test_suite_run_pytest_q_output_mode() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check checks_repo_module_size", "--pytest-q")
    assert proc.returncode in {0, 2}, proc.stderr
    text = proc.stdout
    assert "." in text or "F" in text
    assert "passed" in text and "failed" in text and "skipped" in text


def test_suite_run_text_style_matches_check_style() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check checks_repo_module_size")
    assert proc.returncode in {0, 2}, proc.stderr
    text = proc.stdout
    assert "internal error" not in proc.stderr
    assert text == "" or ("summary: passed=" in text and "failed=" in text and "total=" in text) or ("FAIL check " in text)


def test_suite_run_profile_and_slow_report(tmp_path) -> None:
    target = tmp_path / "suite-profile-out"
    slow_report = tmp_path / "suite-slow.json"
    proc = run_atlasctl(
        "--quiet",
        "suite",
        "run",
        "fast",
        "--only",
        "check checks_repo_module_size",
        "--json",
        "--target-dir",
        str(target),
        "--profile",
        "--slow-threshold-ms",
        "1",
        "--slow-report",
        str(slow_report),
    )
    assert proc.returncode in {0, 2}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["slow_threshold_ms"] == 1
    assert "slow_checks" in payload
    assert (target / "profile.json").exists()
    assert slow_report.exists()


def test_suite_coverage_json() -> None:
    proc = run_atlasctl("--quiet", "suite", "coverage", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    assert "checks_repo_module_size" in payload["coverage"]


def test_suite_run_dry_run() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "required", "--dry-run", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["dry_run"] is True
    assert payload["command"] == "suite"


def test_suite_list_by_group() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--by-group", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert "required" in payload["by_group"]
    assert "slow" in payload["by_group"]


def test_internal_suite_requires_env_gate() -> None:
    proc = run_atlasctl("--quiet", "suite", "internal", "--list", "--json")
    assert proc.returncode == 2, proc.stderr


def test_suites_do_not_include_legacy_checks() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert not any("legacy-path checks" in err for err in payload.get("errors", []))


def test_suite_lane_alias_ci_fast_dry_run_text_lists_check_ids() -> None:
    proc = run_atlasctl("--quiet", "--format", "text", "suite", "run", "ci:fast", "--list")
    assert proc.returncode == 0, proc.stderr
    rows = [ln.strip() for ln in proc.stdout.splitlines() if ln.strip()]
    assert rows
    assert rows[0].startswith("checks_")
