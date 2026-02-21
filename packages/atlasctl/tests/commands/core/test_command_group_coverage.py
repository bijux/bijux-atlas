from __future__ import annotations

from tests.helpers import ROOT


REQUIRED_GROUPS = ('docs', 'ops', 'policies', 'dev', 'core')


def test_every_command_group_has_tests() -> None:
    base = ROOT / 'packages/atlasctl/tests/commands'
    missing = []
    for group in REQUIRED_GROUPS:
        group_dir = base / group
        files = sorted(group_dir.glob('test_*.py')) if group_dir.exists() else []
        if not files:
            missing.append(group)
    assert not missing, f'command groups missing tests: {missing}'
