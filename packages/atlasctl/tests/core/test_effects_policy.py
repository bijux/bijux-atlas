from __future__ import annotations

from atlasctl.cli.surface_registry import command_registry
from atlasctl.core.effects import command_effects, command_group, group_allowed_effects
from atlasctl.network_guard import resolve_network_mode


def test_registry_commands_have_declared_effects_within_group_policy() -> None:
    for spec in command_registry():
        effects = command_effects(spec.name)
        assert effects, f"{spec.name} has no declared effects"
        allowed = set(group_allowed_effects(command_group(spec.name)))
        assert set(effects).issubset(allowed), f"{spec.name} effects {effects} exceed allowed {sorted(allowed)}"


def test_docs_and_policies_groups_default_no_network() -> None:
    for cmd in ("docs", "policies"):
        decision = resolve_network_mode(
            command_name=cmd,
            requested_allow_network=False,
            explicit_network=None,
            deprecated_no_network=False,
        )
        assert decision.mode == "forbid"
        denied = resolve_network_mode(
            command_name=cmd,
            requested_allow_network=True,
            explicit_network=None,
            deprecated_no_network=False,
        )
        assert denied.mode == "forbid"
        assert denied.allow_effective is False


def test_ops_group_can_enable_network_via_allow_switch() -> None:
    denied = resolve_network_mode(
        command_name="ops",
        requested_allow_network=False,
        explicit_network=None,
        deprecated_no_network=False,
    )
    assert denied.mode == "forbid"
    allowed = resolve_network_mode(
        command_name="ops",
        requested_allow_network=True,
        explicit_network=None,
        deprecated_no_network=False,
    )
    assert allowed.mode == "allow"
    assert allowed.allow_effective is True
