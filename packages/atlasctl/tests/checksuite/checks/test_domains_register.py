from __future__ import annotations

from atlasctl.checks.domains import register_all


def test_register_all_is_sorted_and_unique() -> None:
    checks = register_all()
    ids = [str(check.check_id) for check in checks]
    assert ids == sorted(ids)
    assert len(ids) == len(set(ids))


def test_register_all_contains_root_make_python_domains() -> None:
    checks = register_all()
    domains = {str(check.domain) for check in checks}
    assert "make" in domains
    assert "python" in domains
    assert any("root" in str(check.check_id) for check in checks)
