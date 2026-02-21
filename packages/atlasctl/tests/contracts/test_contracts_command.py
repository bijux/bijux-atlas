from __future__ import annotations

import json
from pathlib import Path

from helpers import run_atlasctl

ROOT = Path(__file__).resolve().parents[4]


def test_contracts_list_json_matches_golden() -> None:
    # schema-validate-exempt: contracts list payload does not have a dedicated schema yet.
    proc = run_atlasctl("--quiet", "contracts", "list", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    golden = (ROOT / "packages/atlasctl/tests/goldens/contracts-list.json.golden").read_text(encoding="utf-8").strip()
    assert json.loads(proc.stdout) == json.loads(golden)


def test_contracts_lint_json_passes() -> None:
    proc = run_atlasctl("--quiet", "contracts", "lint", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
