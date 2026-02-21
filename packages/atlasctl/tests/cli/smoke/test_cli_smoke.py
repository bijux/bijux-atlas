from __future__ import annotations

import json

import pytest

from tests.helpers import run_atlasctl_isolated


@pytest.mark.unit
def test_cli_smoke_help_version_self_check(tmp_path) -> None:
    help_proc = run_atlasctl_isolated(tmp_path, "--json", "help")
    assert help_proc.returncode == 0, help_proc.stderr
    help_payload = json.loads(help_proc.stdout)
    assert help_payload["status"] == "ok"

    version_proc = run_atlasctl_isolated(tmp_path, "--json", "version")
    assert version_proc.returncode == 0, version_proc.stderr
    version_payload = json.loads(version_proc.stdout)
    assert version_payload["tool"] == "atlasctl"

    self_check_proc = run_atlasctl_isolated(tmp_path, "--json", "self-check")
    assert self_check_proc.returncode in (0, 3), self_check_proc.stderr
