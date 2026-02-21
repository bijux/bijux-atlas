from __future__ import annotations

# schema-validate-exempt: docs --list payload has no dedicated schema contract yet.
import json

from tests.helpers import golden_text, run_atlasctl


def test_docs_list_json_matches_golden() -> None:
    proc = run_atlasctl("--quiet", "docs", "--list", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    expected = json.loads(golden_text("docs-list.json.golden"))
    assert payload == expected
