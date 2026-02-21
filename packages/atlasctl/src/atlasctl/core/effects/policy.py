from __future__ import annotations

from dataclasses import dataclass

EffectName = str
CommandGroup = str

_COMMAND_GROUPS: dict[str, CommandGroup] = {
    "docs": "docs",
    "configs": "configs",
    "policies": "policies",
    "ops": "ops",
    "k8s": "ops",
    "stack": "ops",
    "load": "ops",
    "obs": "ops",
    "doctor": "dev",
    "inventory": "dev",
    "gates": "dev",
    "repo": "dev",
    "report": "dev",
    "suite": "dev",
    "lint": "dev",
    "contracts": "dev",
    "check": "dev",
    "test": "dev",
    "registry": "dev",
    "layout": "dev",
    "list": "dev",
    "make": "dev",
    "run-id": "dev",
    "install": "dev",
    "release": "dev",
    "dev": "dev",
    "internal": "internal",
}

_GROUP_ALLOWED_EFFECTS: dict[CommandGroup, tuple[EffectName, ...]] = {
    "docs": ("read", "write", "process"),
    "configs": ("read", "write", "process"),
    "dev": ("read", "write", "process"),
    "ops": ("read", "write", "process", "network"),
    "policies": ("read", "write", "process"),
    "internal": ("read", "write", "process"),
}

_COMMAND_EFFECTS: dict[str, tuple[EffectName, ...]] = {
    "docs": ("read", "write", "process"),
    "configs": ("read", "write", "process"),
    "policies": ("read", "write", "process"),
    "ops": ("read", "write", "process", "network"),
    "k8s": ("read", "write", "process", "network"),
    "stack": ("read", "write", "process", "network"),
    "load": ("read", "write", "process", "network"),
    "obs": ("read", "write", "process", "network"),
    "doctor": ("read", "write", "process"),
    "inventory": ("read", "write", "process"),
    "gates": ("read", "write", "process"),
    "repo": ("read", "write", "process"),
    "report": ("read", "write", "process"),
    "suite": ("read", "write", "process"),
    "lint": ("read", "write", "process"),
    "contracts": ("read", "write", "process"),
    "check": ("read", "write", "process"),
    "test": ("read", "write", "process"),
    "registry": ("read", "write", "process"),
    "layout": ("read", "write", "process"),
    "list": ("read", "write", "process"),
    "make": ("read", "write", "process"),
    "run-id": ("read", "write", "process"),
    "install": ("read", "write", "process"),
    "release": ("read", "write", "process"),
    "dev": ("read", "write", "process"),
    "internal": ("read", "write", "process"),
}


@dataclass(frozen=True)
class NetworkDecision:
    mode: str
    group: CommandGroup
    reason: str
    allow_requested: bool
    allow_effective: bool


def command_group(command_name: str) -> CommandGroup:
    return _COMMAND_GROUPS.get(command_name, "internal")


def group_allowed_effects(group: CommandGroup) -> tuple[EffectName, ...]:
    return _GROUP_ALLOWED_EFFECTS.get(group, ("read", "process"))


def command_effects(command_name: str) -> tuple[EffectName, ...]:
    return _COMMAND_EFFECTS.get(command_name, ("read", "process"))


def all_command_effects() -> dict[str, tuple[EffectName, ...]]:
    return dict(_COMMAND_EFFECTS)


def resolve_network_mode(
    *,
    command_name: str,
    requested_allow_network: bool,
    explicit_network: str | None,
    deprecated_no_network: bool,
) -> NetworkDecision:
    group = command_group(command_name)
    allowed = "network" in group_allowed_effects(group)
    if deprecated_no_network:
        return NetworkDecision("forbid", group, "deprecated_no_network_flag", requested_allow_network, False)
    if explicit_network == "forbid":
        return NetworkDecision("forbid", group, "explicit_forbid", requested_allow_network, False)
    if explicit_network == "allow":
        if not allowed:
            return NetworkDecision("forbid", group, "group_forbidden_explicit_allow_denied", requested_allow_network, False)
        return NetworkDecision("allow", group, "explicit_allow", requested_allow_network, True)
    if requested_allow_network:
        if not allowed:
            return NetworkDecision("forbid", group, "group_forbidden_allow_network_denied", requested_allow_network, False)
        return NetworkDecision("allow", group, "allow_network_flag", requested_allow_network, True)
    return NetworkDecision("forbid", group, "default_deny", requested_allow_network, False)
