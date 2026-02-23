from __future__ import annotations

from atlasctl.registry.spine import load_registry


def test_registry_spine_loads_with_stable_ordering() -> None:
    reg = load_registry()
    assert reg.commands == tuple(sorted(reg.commands, key=lambda c: (c.group, c.name)))
    assert reg.checks == tuple(sorted(reg.checks, key=lambda c: (c.domain, c.category, c.check_id)))


def test_registry_spine_selectors_work() -> None:
    reg = load_registry()
    repo_checks = reg.select_checks(domain="repo")
    assert all(item.domain == "repo" for item in repo_checks)
    platform_cmds = reg.select_commands(tags=("artifacts/evidence/",))
    assert platform_cmds


def test_registry_spine_includes_budget_and_capabilities() -> None:
    reg = load_registry()
    assert reg.budgets.defaults
    assert any(cap.subject_kind == "command" for cap in reg.capabilities)
    assert any(cap.subject_kind == "check" for cap in reg.capabilities)
