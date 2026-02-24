from __future__ import annotations

from pathlib import Path


def test_check_command_does_not_import_docs_domain_mirror() -> None:
    run_py = Path("packages/atlasctl/src/atlasctl/commands/check/run.py")
    text = run_py.read_text(encoding="utf-8")
    assert "checks.tools.docs_domain" not in text


def test_check_command_does_not_import_checks_runner_module() -> None:
    run_py = Path("packages/atlasctl/src/atlasctl/commands/check/run.py")
    command_py = Path("packages/atlasctl/src/atlasctl/commands/check/command.py")
    assert "checks.runner" not in run_py.read_text(encoding="utf-8")
    assert "checks.runner" not in command_py.read_text(encoding="utf-8")
