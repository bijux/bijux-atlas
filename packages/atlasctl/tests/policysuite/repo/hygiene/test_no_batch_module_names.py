from __future__ import annotations

from pathlib import Path

from tests.helpers import ROOT


def test_repo_tests_use_intent_names_not_batch_prefix() -> None:
    repo_tests = ROOT / 'packages/atlasctl/tests/repo'
    offenders = sorted(path.relative_to(ROOT).as_posix() for path in repo_tests.rglob('test_*.py') if path.name.startswith('test_batch'))
    assert not offenders, f'batch-prefixed test modules are forbidden: {offenders}'
