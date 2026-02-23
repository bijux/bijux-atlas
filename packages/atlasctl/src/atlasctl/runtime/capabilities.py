from __future__ import annotations

from dataclasses import dataclass

from atlasctl.cli.surface_registry import CommandSpec, command_spec


@dataclass(frozen=True)
class CommandCapabilities:
    command: str
    tools: tuple[str, ...]
    touches: tuple[str, ...]
    effect_level: str
    network_allowed: bool
    writes_allowed_roots: tuple[str, ...]


def build_command_capabilities(spec: CommandSpec) -> CommandCapabilities:
    touches = tuple(spec.touches)
    network_allowed = any(tool in {"curl", "wget"} for tool in spec.tools) or spec.name in {"ops", "docker"}
    return CommandCapabilities(
        command=spec.name,
        tools=tuple(spec.tools),
        touches=touches,
        effect_level=str(spec.effect_level),
        network_allowed=network_allowed,
        writes_allowed_roots=touches,
    )


def capabilities_for_command(command: str) -> CommandCapabilities | None:
    spec = command_spec(command)
    if spec is None:
        return None
    return build_command_capabilities(spec)


def validate_command_capabilities(command: str) -> tuple[bool, str]:
    caps = capabilities_for_command(command)
    if caps is None:
        return False, f"no command spec/capability manifest entry for `{command}`"
    if caps.effect_level == "effectful" and not caps.writes_allowed_roots:
        return False, f"effectful command `{command}` must declare touches/write roots in capability manifest"
    return True, "ok"


__all__ = ["CommandCapabilities", "build_command_capabilities", "capabilities_for_command", "validate_command_capabilities"]
