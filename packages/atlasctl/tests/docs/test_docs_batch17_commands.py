from __future__ import annotations

import json

from helpers import run_atlasctl


def test_docs_validate_command_json() -> None:
    proc = run_atlasctl("--quiet", "docs", "validate", "--report", "json", "--fail-fast")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}


def test_docs_generate_command_groups_json() -> None:
    proc = run_atlasctl("--quiet", "docs", "generate-command-groups-docs", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["status"] in {"pass", "fail"}
