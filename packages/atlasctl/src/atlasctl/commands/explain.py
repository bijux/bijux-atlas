"""Command explain metadata powered by CLI registry."""

from __future__ import annotations

from pathlib import Path

from ..checks.registry import get_check
from ..cli.registry import command_registry
from ..suite.manifests import load_first_class_suites


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


def describe_thing(repo_root: Path, thing: str) -> dict[str, object]:
    check = get_check(thing)
    if check is not None:
        return {
            "kind": "check",
            "name": check.check_id,
            "contract": "atlasctl.check-list.v1",
            "purpose": check.title,
            "examples": [f"atlasctl check run --select atlasctl::{check.domain}::{check.check_id}"],
            "touches": [],
            "tools": [],
            "tags": list(check.tags),
        }
    suites = load_first_class_suites()
    suite = suites.get(thing)
    if suite is not None:
        return {
            "kind": "suite",
            "name": suite.name,
            "contract": "atlasctl.suite-run.v1",
            "purpose": f"first-class suite `{suite.name}`",
            "examples": [f"atlasctl suite {suite.name} --list --json", f"atlasctl run suite {suite.name} --json"],
            "touches": [],
            "tools": [],
            "required_env": list(suite.required_env),
            "default_effects": list(suite.default_effects),
        }
    return {"kind": "command", "name": thing, **describe_command(thing)}
