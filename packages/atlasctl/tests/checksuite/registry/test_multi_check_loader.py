from __future__ import annotations

import types

from atlasctl.checks.api import check
from atlasctl.checks.registry_legacy.catalog import _from_entry
from atlasctl.checks.registry_legacy.ssot import RegistryEntry


def test_loader_resolves_check_from_multi_check_module(monkeypatch) -> None:  # type: ignore[no-untyped-def]
    module = types.ModuleType("atlasctl.checks.domains.repo.multi_demo")

    @check(check_id="checks_repo_multi_demo", domain="repo", intent="validate multi-check loader")
    def demo(_repo_root):  # type: ignore[no-untyped-def]
        return 0, []

    module.CHECKS = [demo]
    monkeypatch.setattr("importlib.import_module", lambda _name: module)

    entry = RegistryEntry(
        id="checks_repo_multi_demo",
        domain="repo",
        area="multi",
        gate="repo",
        owner="platform",
        speed="fast",
        groups=("repo",),
        timeout_ms=1000,
        module="atlasctl.checks.domains.repo.multi_demo",
        callable="CHECKS",
        description="multi loader",
        category="check",
    )
    loaded = _from_entry(
        entry,
        module_callable_index={(entry.module, demo.__name__): demo},
        module_check_id_index={(entry.module, entry.id): demo},
        legacy_by_id={},
    )
    code, errors = loaded.fn(None)  # type: ignore[arg-type]
    assert code == 0
    assert errors == []
