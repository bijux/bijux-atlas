from __future__ import annotations

from atlasctl.checks.registry import list_checks


def test_registry_check_ids_are_unique() -> None:
    ids = [str(check.check_id) for check in list_checks()]
    assert len(ids) == len(set(ids))


def test_registry_domain_segment_matches_check_domain_for_canonical_ids() -> None:
    for check in list_checks():
        check_id = str(check.check_id)
        if not check_id.startswith("checks_"):
            continue
        parts = check_id.split("_", 2)
        assert len(parts) == 3
        assert parts[1] == str(check.domain)
