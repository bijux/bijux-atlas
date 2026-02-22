"""Command explain metadata powered by CLI registry."""

from __future__ import annotations

import inspect
from pathlib import Path

from ..checks.registry import get_check
from ..cli.surface_registry import command_registry
from .dev.make.public_targets import entry_map, load_ownership
from .policies.runtime.command import _POLICIES_ITEMS
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
    if thing.startswith("policy:"):
        policy_name = thing.split(":", 1)[1]
        if policy_name in _POLICIES_ITEMS:
            return {
                "kind": "policy",
                "name": policy_name,
                "contract": "atlasctl.policy.v1",
                "purpose": "policy command surface",
                "examples": [
                    f"atlasctl policies {policy_name} --report json" if policy_name != "check" else "atlasctl policies check --report json",
                    f"atlasctl policies explain {policy_name}",
                ],
                "touches": ["configs/policy/", "artifacts/reports/atlasctl/"],
                "tools": [],
            }
        return {
            "kind": "policy",
            "name": policy_name,
            "contract": "atlasctl.policy.v1",
            "purpose": "",
            "examples": ["atlasctl policies --list", "atlasctl list policies --json"],
            "touches": ["configs/policy/"],
            "tools": [],
            "note": "unknown policy command",
        }
    if thing.startswith("make:"):
        target = thing.split(":", 1)[1]
        entries = entry_map()
        if target in entries:
            ownership = load_ownership().get(target, {})
            entry = entries[target]
            return {
                "kind": "make-target",
                "name": target,
                "contract": "atlasctl.make.surface.v1",
                "purpose": entry.get("description", ""),
                "examples": [f"make {target}", f"atlasctl make explain {target}", f"atlasctl make run {target} --json"],
                "touches": ["makefiles/root.mk", "configs/make/public-targets.json"],
                "tools": ["make", "atlasctl"],
                "owner": ownership.get("owner", ""),
                "area": ownership.get("area", entry.get("area", "")),
            }
        return {
            "kind": "make-target",
            "name": target,
            "contract": "atlasctl.make.surface.v1",
            "purpose": "",
            "examples": ["atlasctl make list-public-targets --json"],
            "touches": ["configs/make/public-targets.json"],
            "tools": ["atlasctl"],
            "note": "unknown public make target",
        }
    check = get_check(thing)
    if check is not None:
        source = inspect.getsourcefile(check.fn)
        source_rel = None
        if source:
            source_path = Path(source).resolve()
            try:
                source_rel = source_path.relative_to(repo_root).as_posix()
            except ValueError:
                source_rel = source_path.as_posix()
        return {
            "kind": "check",
            "name": check.check_id,
            "contract": "atlasctl.check-list.v1",
            "purpose": check.title,
            "examples": [f"atlasctl check run atlasctl::{check.domain}::{check.check_id}"],
            "touches": [],
            "tools": [],
            "tags": list(check.tags),
            "owners": list(check.owners),
            "source": source_rel,
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
