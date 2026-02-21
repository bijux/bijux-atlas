from __future__ import annotations

import json

from helpers import run_atlasctl


def test_suite_list_json() -> None:
    proc = run_atlasctl("--quiet", "suite", "list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["default"] == "refgrade"
    names = [item["name"] for item in payload["suites"]]
    assert {"fast", "refgrade", "ops", "ci", "refgrade_proof"}.issubset(set(names))


def test_suite_run_list_refgrade() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "refgrade", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "refgrade"
    assert payload["total_count"] >= 1
    assert any(item.startswith("check:") for item in payload["tasks"])


def test_suite_run_only_single_check() -> None:
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check repo.module_size", "--json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["suite"] == "fast"
    assert payload["summary"]["passed"] + payload["summary"]["failed"] >= 1
    assert payload["results"][0]["label"] == "check repo.module_size"


def test_suite_inventory_check_json() -> None:
    proc = run_atlasctl("--quiet", "suite", "check", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"


def test_suite_run_writes_target_dir(tmp_path) -> None:
    target = tmp_path / "suite-out"
    proc = run_atlasctl("--quiet", "suite", "run", "fast", "--only", "check repo.module_size", "--json", "--target-dir", str(target))
    assert proc.returncode in {0, 1}, proc.stderr
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
    assert payload1["tasks"] == payload2["tasks"]
    tasks = set(payload1["tasks"])
    assert "cmd:atlasctl test run unit" in tasks
    assert "cmd:atlasctl policies check --report json" in tasks
    assert len([task for task in tasks if "atlasctl test run" in task]) == 1
