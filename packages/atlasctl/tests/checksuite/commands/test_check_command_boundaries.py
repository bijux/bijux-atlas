from __future__ import annotations

from pathlib import Path


def test_check_command_does_not_import_docs_domain_mirror() -> None:
    run_py = Path("packages/atlasctl/src/atlasctl/commands/check/run.py")
    text = run_py.read_text(encoding="utf-8")
    assert "checks.tools.docs_domain" not in text
