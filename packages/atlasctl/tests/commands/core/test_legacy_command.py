from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_legacy_inventory_flag_outputs_payload() -> None:
    proc = run_atlasctl("--quiet", "legacy", "--inventory", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "inventory"
    assert "legacy_concepts" in payload
    assert isinstance(payload["references"], list)


def test_legacy_inventory_default_action() -> None:
    proc = run_atlasctl("--quiet", "legacy", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "inventory"
