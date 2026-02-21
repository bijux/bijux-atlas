from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_dev_split_module_json() -> None:
    proc = run_atlasctl("dev", "split-module", "--path", "packages/atlasctl/src/atlasctl/dev/command.py", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "ok"
    assert "split_plan" in payload
    assert payload["recommended_doc"].endswith("how-to-split-a-module.md")
