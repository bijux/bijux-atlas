"""Command explain metadata powered by CLI registry."""

from __future__ import annotations

from ..cli.registry import command_registry


def describe_command(name: str) -> dict[str, object]:
    for spec in command_registry():
        if spec.name == name:
            return {
                "contract": "atlasctl.commands.v1",
                "purpose": spec.purpose or spec.help_text,
                "examples": list(spec.examples),
                "touches": list(spec.touches),
                "tools": list(spec.tools),
                "failure_modes": list(spec.failure_modes),
                "effect_level": spec.effect_level,
                "run_id_mode": spec.run_id_mode,
                "supports_dry_run": spec.supports_dry_run,
                "aliases": list(spec.aliases),
            }
    return {"contract": "atlasctl.commands.v1", "purpose": "", "examples": [], "touches": [], "tools": [], "note": "unknown command", "aliases": []}
