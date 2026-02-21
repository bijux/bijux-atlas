from __future__ import annotations

import json
from pathlib import Path

from helpers import run_atlasctl


def test_runtime_contracts_check_writes_artifact(tmp_path: Path) -> None:
    out = tmp_path / "contracts" / "runtime-contracts.json"
    proc = run_atlasctl("--json", "check", "runtime-contracts", "--out-file", str(out), evidence_root=tmp_path)
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert out.exists()


def test_check_list_includes_metadata_fields() -> None:
    proc = run_atlasctl("--json", "check", "list")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    first = payload["checks"][0]
    for key in ("id", "domain", "description", "severity", "category", "fix_hint", "slow", "external_tools"):
        assert key in first
