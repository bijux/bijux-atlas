from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]


def test_no_legacy_observability_or_reporting_namespace() -> None:
    legacy_root = ROOT / "packages/atlasctl/src/atlasctl/legacy"
    assert not legacy_root.exists()
    assert not (ROOT / "packages/atlasctl/src/atlasctl/report").exists()


def test_suite_runner_has_single_canonical_command() -> None:
    suite_command = ROOT / "packages/atlasctl/src/atlasctl/suite/command.py"
    lint_runner = ROOT / "packages/atlasctl/src/atlasctl/lint/runner.py"
    assert "def run_suite_command(" in suite_command.read_text(encoding="utf-8")
    assert "def run_lint_suite(" in lint_runner.read_text(encoding="utf-8")
    assert "def run_suite(" not in lint_runner.read_text(encoding="utf-8")


def test_check_runner_uses_shared_execution_engine() -> None:
    check_command = ROOT / "packages/atlasctl/src/atlasctl/commands/check/command.py"
    text = check_command.read_text(encoding="utf-8")
    assert "run_function_checks(" in text
    assert "check.fn(ctx.repo_root)" not in text
