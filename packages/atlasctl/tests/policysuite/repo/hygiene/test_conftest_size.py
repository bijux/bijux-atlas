from __future__ import annotations

from tests.helpers import ROOT


def test_conftest_stays_small() -> None:
    conftest = ROOT / 'packages/atlasctl/tests/conftest.py'
    lines = conftest.read_text(encoding='utf-8').splitlines()
    assert len(lines) <= 120, f'conftest.py too large: {len(lines)} lines (max 120)'
