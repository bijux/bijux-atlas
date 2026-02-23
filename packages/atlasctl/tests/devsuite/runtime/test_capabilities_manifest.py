from __future__ import annotations

from atlasctl.runtime.capabilities import capabilities_for_command, validate_command_capabilities


def test_capabilities_for_known_command() -> None:
    caps = capabilities_for_command('ops')
    assert caps is not None
    assert caps.command == 'ops'
    assert 'ops/' in caps.writes_allowed_roots


def test_validate_unknown_command_fails() -> None:
    ok, msg = validate_command_capabilities('definitely-not-a-command')
    assert not ok
    assert 'no command spec' in msg
