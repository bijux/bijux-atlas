from __future__ import annotations

import json

from helpers import run_atlasctl


def test_test_inventory_json() -> None:
    proc = run_atlasctl("--quiet", "test", "inventory", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "ok"
    assert payload["total_tests"] >= 1
    assert any(row["domain"] == "cli" for row in payload["domains"])


def test_test_refresh_goldens_json() -> None:
    proc = run_atlasctl("--quiet", "test", "refresh-goldens", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout.splitlines()[0])
    assert payload["status"] == "ok"
