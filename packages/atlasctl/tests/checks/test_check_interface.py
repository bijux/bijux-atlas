from __future__ import annotations

from atlasctl.checks.registry import check_tags, list_checks


def test_registered_checks_expose_batch6_interface() -> None:
    checks = list_checks()
    assert checks
    for check in checks:
        assert check.id == check.check_id
        assert check.title == check.description
        assert isinstance(check.effects, tuple)
        assert isinstance(check.owners, tuple)


def test_registered_checks_have_stable_control_plane_tags() -> None:
    allowed = {"docs", "dev", "ops", "policies", "configs", "internal"}
    checks = list_checks()
    assert checks
    for check in checks:
        tags = set(check_tags(check))
        assert tags & allowed, f"missing stable control-plane tag for {check.check_id}: {sorted(tags)}"
