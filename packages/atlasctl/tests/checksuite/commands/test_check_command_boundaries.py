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


def test_python_sources_do_not_import_docs_domain_integrity_module() -> None:
    src_root = Path("packages/atlasctl/src/atlasctl")
    for path in src_root.rglob("*.py"):
        text = path.read_text(encoding="utf-8")
        assert "checks.tools.docs_domain.integrity" not in text, path.as_posix()


def test_check_command_uses_deterministic_check_evidence_root() -> None:
    command_py = Path("packages/atlasctl/src/atlasctl/commands/check/command.py")
    text = command_py.read_text(encoding="utf-8")
    assert 'CHECK_EVIDENCE_ROOT = "artifacts/atlasctl/check"' in text


def test_check_command_blocks_running_from_ops_cwd() -> None:
    command_py = Path("packages/atlasctl/src/atlasctl/commands/check/command.py")
    text = command_py.read_text(encoding="utf-8")
    assert "refusing to run checks from inside ops/" in text


def test_check_command_supports_lint_suite_alias() -> None:
    command_py = Path("packages/atlasctl/src/atlasctl/commands/check/command.py")
    text = command_py.read_text(encoding="utf-8")
    assert 'if suite_name == "lint"' in text


def test_check_command_uses_checks_engine_for_registry_runs() -> None:
    command_py = Path("packages/atlasctl/src/atlasctl/commands/check/command.py")
    text = command_py.read_text(encoding="utf-8")
    assert "run_checks_engine(" in text
    assert "run_checks_payload(" not in text
