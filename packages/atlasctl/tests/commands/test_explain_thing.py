from __future__ import annotations

import json

from helpers import run_atlasctl


def test_explain_supports_suite_and_check_targets() -> None:
    suite_proc = run_atlasctl("--quiet", "--json", "explain", "docs")
    check_proc = run_atlasctl("--quiet", "--json", "explain", "docs.index_complete")
    assert suite_proc.returncode == 0, suite_proc.stderr
    assert check_proc.returncode == 0, check_proc.stderr
    suite_payload = json.loads(suite_proc.stdout)
    check_payload = json.loads(check_proc.stdout)
    assert suite_payload["contract"] == "atlasctl.suite-run.v1"
    assert check_payload["contract"] == "atlasctl.check-list.v1"
