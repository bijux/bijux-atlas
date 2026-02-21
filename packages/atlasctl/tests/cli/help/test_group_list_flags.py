from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_major_groups_support_list_output() -> None:
    for group in ("docs", "ops", "dev", "policies", "configs", "internal"):
        proc = run_atlasctl("--quiet", group, "--list")
        assert proc.returncode == 0, f"{group}: {proc.stderr}"
        assert proc.stdout.strip(), group


def test_major_groups_support_list_json() -> None:
    for group in ("docs", "ops", "dev", "policies", "configs", "internal"):
        proc = run_atlasctl("--quiet", group, "--list", "--json")
        assert proc.returncode == 0, f"{group}: {proc.stderr}"
        payload = json.loads(proc.stdout)
        assert payload["status"] == "ok"
        assert payload["group"] == group
        assert payload["items"]


def test_check_and_suite_top_level_list() -> None:
    check_proc = run_atlasctl("--quiet", "check", "--list")
    assert check_proc.returncode == 0, check_proc.stderr
    assert "repo.module_size" in check_proc.stdout

    suite_proc = run_atlasctl("--quiet", "suite", "--list")
    assert suite_proc.returncode == 0, suite_proc.stderr
    lines = [line.strip() for line in suite_proc.stdout.splitlines() if line.strip()]
    assert "ci" in lines
