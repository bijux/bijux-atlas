from __future__ import annotations

import json
from pathlib import Path

from .....cli.surface_registry import command_registry
from .....core.meta.owners import load_owner_catalog
from .....core.effects import all_command_effects, command_effects, command_group, group_allowed_effects, resolve_network_mode
from .....contracts.ids import RUNTIME_CONTRACTS

_PUBLIC_UNSTABLE_ALLOWLIST: set[str] = set()
_ALLOWED_EFFECT_LEVELS = {"pure", "effectful"}
_ALLOWED_RUN_ID_MODES = {"not_required", "accept_or_generate"}
_MAX_ALIASES_PER_COMMAND = 1


def check_command_metadata_contract(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    declared = all_command_effects()
    owner_catalog = load_owner_catalog(repo_root)
    known_owners = set(owner_catalog.owners)
    for spec in command_registry():
        if not spec.touches:
            errors.append(f"{spec.name}: missing touches metadata")
        if spec.tools is None:
            errors.append(f"{spec.name}: missing tools metadata")
        if not spec.owner:
            errors.append(f"{spec.name}: missing owner metadata")
        elif spec.owner not in known_owners:
            errors.append(f"{spec.name}: owner `{spec.owner}` is not declared in configs/meta/owners.json")
        if not spec.doc_link:
            errors.append(f"{spec.name}: missing doc_link metadata")
        if spec.effect_level not in _ALLOWED_EFFECT_LEVELS:
            errors.append(f"{spec.name}: invalid effect_level={spec.effect_level}")
        if spec.run_id_mode not in _ALLOWED_RUN_ID_MODES:
            errors.append(f"{spec.name}: invalid run_id_mode={spec.run_id_mode}")
        if spec.effect_level == "effectful" and spec.run_id_mode != "accept_or_generate":
            errors.append(f"{spec.name}: effectful commands must accept or generate run_id")
        if spec.effect_level == "effectful" and not spec.supports_dry_run:
            errors.append(f"{spec.name}: effectful commands must support dry-run")
        if len(spec.aliases) > _MAX_ALIASES_PER_COMMAND:
            errors.append(f"{spec.name}: alias budget exceeded ({len(spec.aliases)} > {_MAX_ALIASES_PER_COMMAND})")
        if spec.aliases != tuple(sorted(spec.aliases)):
            errors.append(f"{spec.name}: aliases must be sorted")
        if not (spec.purpose or "").strip():
            errors.append(f"{spec.name}: missing purpose metadata")
        if not spec.examples:
            errors.append(f"{spec.name}: missing examples metadata")
        if spec.name not in declared:
            errors.append(f"{spec.name}: missing per-command effects declaration")
            continue
        effects = command_effects(spec.name)
        if not effects:
            errors.append(f"{spec.name}: empty effects declaration")
        group = command_group(spec.name)
        allowed = set(group_allowed_effects(group))
        unknown = [effect for effect in effects if effect not in allowed]
        if unknown:
            errors.append(f"{spec.name}: effects {unknown} violate allowed group effects for {group} ({sorted(allowed)})")
        if "network" in effects and group in {"docs", "policies"}:
            errors.append(f"{spec.name}: docs/policies command cannot declare network effect")
    if errors:
        return 1, errors
    return 0, []


def check_command_group_owners(repo_root: Path) -> tuple[int, list[str]]:
    catalog = load_owner_catalog(repo_root)
    known_owners = set(catalog.owners)
    mapping = dict(catalog.command_groups)
    errors: list[str] = []
    seen_groups: set[str] = set()
    for spec in command_registry():
        group = spec.name.split(" ", 1)[0]
        seen_groups.add(group)
        mapped_owner = mapping.get(group)
        if not mapped_owner:
            errors.append(f"{group}: missing command_groups owner mapping in configs/meta/owners.json")
            continue
        if mapped_owner not in known_owners:
            errors.append(f"{group}: mapped owner `{mapped_owner}` is unknown")
            continue
        if mapped_owner != spec.owner:
            errors.append(f"{group}: owner mismatch (registry={spec.owner}, owners.json={mapped_owner})")
    extra = sorted(set(mapping) - seen_groups)
    for group in extra:
        errors.append(f"{group}: owners.json command_groups entry has no matching command group")
    return (0 if not errors else 1), errors


def check_network_default_deny_policy(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for spec in command_registry():
        default = resolve_network_mode(
            command_name=spec.name,
            requested_allow_network=False,
            explicit_network=None,
            deprecated_no_network=False,
        )
        if default.allow_effective:
            errors.append(f"{spec.name}: network must be denied by default")
        requested = resolve_network_mode(
            command_name=spec.name,
            requested_allow_network=True,
            explicit_network=None,
            deprecated_no_network=False,
        )
        declared = set(command_effects(spec.name))
        if "network" not in declared and requested.allow_effective:
            errors.append(f"{spec.name}: --allow-network enabled without declared network effect")
    return (0 if not errors else 1), errors


def check_no_duplicate_command_names(repo_root: Path) -> tuple[int, list[str]]:
    seen: set[str] = set()
    dupes: list[str] = []
    for spec in command_registry():
        if spec.name in seen:
            dupes.append(spec.name)
        seen.add(spec.name)
    if dupes:
        return 1, [f"duplicate command names: {', '.join(sorted(set(dupes)))}"]
    return 0, []


def check_internal_commands_not_public(repo_root: Path) -> tuple[int, list[str]]:
    docs_cli = repo_root / "docs/_generated/cli.md"
    if not docs_cli.exists():
        return 1, ["docs/_generated/cli.md missing"]
    text = docs_cli.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for spec in command_registry():
        if spec.stable:
            continue
        if spec.name in _PUBLIC_UNSTABLE_ALLOWLIST:
            continue
        if f"`{spec.name}`" in text or f"- {spec.name}" in text:
            errors.append(f"internal/unstable command exposed in public docs: {spec.name}")
    return (0 if not errors else 1), errors


def check_command_alias_budget(repo_root: Path) -> tuple[int, list[str]]:
    # Current policy: no command aliases in public command registry.
    names = [spec.name for spec in command_registry()]
    if len(names) != len(set(names)):
        return 1, ["alias/name budget exceeded: duplicate command identifiers present"]
    return 0, []


def check_no_legacy_command_names(repo_root: Path) -> tuple[int, list[str]]:
    offenders = [spec.name for spec in command_registry() if "legacy" in spec.name]
    if offenders:
        return 1, [f"command names must not contain `legacy`: {', '.join(sorted(offenders))}"]
    return 0, []


def check_no_deprecated_commands(repo_root: Path) -> tuple[int, list[str]]:
    deprecated = {"compat", "migration", "migrate", "legacy"}
    offenders = sorted(spec.name for spec in command_registry() if spec.name in deprecated)
    if offenders:
        return 1, [f"deprecated commands must not ship pre-1.0: {', '.join(offenders)}"]
    return 0, []


def check_registry_single_source(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    command_spec_offenders: list[str] = []
    checkdef_offenders: list[str] = []
    allowed_checkdef_paths = {
        "packages/atlasctl/src/atlasctl/checks/domains/__init__.py",
    }
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if ("Command" + "Spec(") in text and rel != "packages/atlasctl/src/atlasctl/cli/surface_registry.py":
            command_spec_offenders.append(rel)
        if rel.startswith("packages/atlasctl/src/atlasctl/checks/domains/") and rel.endswith(".py"):
            allowed_checkdef_paths.add(rel)
        # Transitional allowance while tools mirrors are being migrated into canonical domains.
        if rel.startswith("packages/atlasctl/src/atlasctl/checks/tools/") and rel.endswith("/__init__.py"):
            allowed_checkdef_paths.add(rel)
        if ("Check" + "Def(") in text and "/checks/" in rel and rel not in allowed_checkdef_paths:
            checkdef_offenders.append(rel)
    errors: list[str] = []
    if command_spec_offenders:
        errors.append(f"command registration must be defined only in cli/surface_registry.py: {', '.join(command_spec_offenders)}")
    if checkdef_offenders:
        errors.append(f"check registration must be declared only in checks/domains modules: {', '.join(checkdef_offenders)}")
    return (0 if not errors else 1), errors


def check_command_ownership_docs(repo_root: Path) -> tuple[int, list[str]]:
    ownership_doc = repo_root / "packages/atlasctl/docs/ownership.md"
    if not ownership_doc.exists():
        return 1, ["packages/atlasctl/docs/ownership.md missing"]
    text = ownership_doc.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    owners = sorted({spec.owner for spec in command_registry()})
    for owner in owners:
        if owner not in text:
            errors.append(f"command owner not documented in ownership.md: {owner}")
    return (0 if not errors else 1), errors


def check_command_help_docs_drift(repo_root: Path) -> tuple[int, list[str]]:
    docs_cli = repo_root / "docs/_generated/cli.md"
    if not docs_cli.exists():
        return 1, ["docs/_generated/cli.md missing"]
    text = docs_cli.read_text(encoding="utf-8", errors="ignore")
    listed = {
        line.split("- ", 1)[1].split(" ", 1)[0].strip()
        for line in text.splitlines()
        if line.lstrip().startswith("- ")
    }
    errors: list[str] = []
    for spec in command_registry():
        if spec.internal:
            continue
        if spec.name not in listed:
            errors.append(f"{spec.name}: missing from docs/_generated/cli.md")
    return (0 if not errors else 1), errors


def check_public_commands_docs_index(repo_root: Path) -> tuple[int, list[str]]:
    index = repo_root / "packages/atlasctl/docs/commands/index.md"
    if not index.exists():
        return 1, ["packages/atlasctl/docs/commands/index.md missing"]
    text = index.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for spec in command_registry():
        if spec.internal:
            continue
        if f"`{spec.name}`" not in text:
            errors.append(f"{spec.name}: missing from packages/atlasctl/docs/commands/index.md")
    return (0 if not errors else 1), errors


def check_no_undocumented_help_commands(repo_root: Path) -> tuple[int, list[str]]:
    index = repo_root / "packages/atlasctl/docs/commands/index.md"
    if not index.exists():
        return 1, ["packages/atlasctl/docs/commands/index.md missing"]
    text = index.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for spec in command_registry():
        if spec.internal:
            continue
        if f"`{spec.name}`" not in text:
            errors.append(f"undocumented command appears in help surface: {spec.name}")
    return (0 if not errors else 1), errors


def check_cli_canonical_paths(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    main = repo_root / "packages/atlasctl/src/atlasctl/cli/main.py"
    dispatch = repo_root / "packages/atlasctl/src/atlasctl/cli/dispatch.py"
    output = repo_root / "packages/atlasctl/src/atlasctl/cli/output.py"
    for path in (main, dispatch, output):
        if not path.exists():
            errors.append(f"missing canonical cli path: {path.relative_to(repo_root).as_posix()}")
    if main.exists():
        text = main.read_text(encoding="utf-8", errors="ignore")
        if "dispatch_command(" not in text:
            errors.append("cli/main.py must delegate command execution via dispatch_command")
    if dispatch.exists():
        text = dispatch.read_text(encoding="utf-8", errors="ignore")
        if "emit(" not in text:
            errors.append("cli/dispatch.py must use cli.output.emit for payload output")
    return (0 if not errors else 1), errors


def check_no_duplicate_command_implementation_patterns(repo_root: Path) -> tuple[int, list[str]]:
    src = repo_root / "packages/atlasctl/src/atlasctl"
    commands_dir = src / "commands"
    if not commands_dir.exists():
        return 0, []
    errors: list[str] = []
    for path in sorted(commands_dir.glob("*.py")):
        if path.name == "__init__.py":
            continue
        stem = path.stem
        peer = src / stem / "command.py"
        if peer.exists():
            errors.append(
                "duplicate command implementation pattern detected: "
                f"{path.relative_to(repo_root).as_posix()} and {peer.relative_to(repo_root).as_posix()}"
            )
    return (0 if not errors else 1), errors


def check_command_surface_stability(repo_root: Path) -> tuple[int, list[str]]:
    golden_path = repo_root / "packages/atlasctl/tests/goldens/list/commands.json.golden"
    if not golden_path.exists():
        return 1, [f"{golden_path.relative_to(repo_root).as_posix()} missing"]
    expected_raw = json.loads(golden_path.read_text(encoding="utf-8"))
    expected = {
        "schema_name": expected_raw.get("schema_name", "atlasctl.commands.v1"),
        "schema_version": expected_raw.get("schema_version", 1),
        "tool": expected_raw.get("tool", "atlasctl"),
        "status": expected_raw.get("status", "ok"),
        "run_id": expected_raw.get("run_id", ""),
        "commands": expected_raw.get("commands", []),
    }
    current = {
        "schema_name": "atlasctl.commands.v1",
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "run_id": "",
        "commands": [
            {
                "name": cmd.name,
                "help": cmd.help_text,
                "stable": cmd.stable,
                "touches": list(cmd.touches),
                "tools": list(cmd.tools),
                "failure_modes": list(cmd.failure_modes),
                "owner": cmd.owner,
                "doc_link": cmd.doc_link,
                "effect_level": cmd.effect_level,
                "run_id_mode": cmd.run_id_mode,
                "supports_dry_run": cmd.supports_dry_run,
                "aliases": list(cmd.aliases),
                "internal": cmd.internal,
                "purpose": cmd.purpose or cmd.help_text,
                "examples": list(cmd.examples),
            }
            for cmd in sorted(command_registry(), key=lambda item: item.name)
            if not cmd.internal
        ],
    }
    expected["run_id"] = ""
    if current != expected:
        return 1, ["command surface drift: run `python -m atlasctl.cli gen goldens`"]
    return 0, []


def check_stable_command_no_breaking_changes(repo_root: Path) -> tuple[int, list[str]]:
    golden_path = repo_root / "packages/atlasctl/tests/goldens/list/commands.json.golden"
    if not golden_path.exists():
        return 1, [f"{golden_path.relative_to(repo_root).as_posix()} missing"]
    expected = json.loads(golden_path.read_text(encoding="utf-8"))
    expected_rows = [row for row in expected.get("commands", []) if bool(row.get("stable", False))]
    current_rows = [
        {
            "name": cmd.name,
            "help": cmd.help_text,
            "stable": cmd.stable,
            "touches": list(cmd.touches),
            "tools": list(cmd.tools),
            "failure_modes": list(cmd.failure_modes),
            "owner": cmd.owner,
            "doc_link": cmd.doc_link,
            "effect_level": cmd.effect_level,
            "run_id_mode": cmd.run_id_mode,
            "supports_dry_run": cmd.supports_dry_run,
            "aliases": list(cmd.aliases),
            "internal": cmd.internal,
            "purpose": cmd.purpose or cmd.help_text,
            "examples": list(cmd.examples),
        }
        for cmd in sorted(command_registry(), key=lambda item: item.name)
        if cmd.stable
    ]
    if current_rows != expected_rows:
        return 1, ["stable command contract breaking change detected; update commands golden intentionally"]
    return 0, []


def command_lint_payload(repo_root: Path) -> dict[str, object]:
    checks: list[tuple[str, tuple[int, list[str]]]] = [
        ("command_metadata", check_command_metadata_contract(repo_root)),
        ("duplicate_names", check_no_duplicate_command_names(repo_root)),
        ("docs_index", check_public_commands_docs_index(repo_root)),
        ("help_documented", check_no_undocumented_help_commands(repo_root)),
        ("cli_canonical_paths", check_cli_canonical_paths(repo_root)),
        ("duplicate_impl_patterns", check_no_duplicate_command_implementation_patterns(repo_root)),
        ("surface_stability", check_command_surface_stability(repo_root)),
        ("stable_no_breaking_changes", check_stable_command_no_breaking_changes(repo_root)),
        ("alias_budget", check_command_alias_budget(repo_root)),
    ]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if all(code == 0 for _, (code, _) in checks) else "error",
        "checks": [
            {"id": cid, "status": "ok" if code == 0 else "error", "errors": errors}
            for cid, (code, errors) in checks
        ],
    }


def runtime_contracts_payload(repo_root: Path) -> dict[str, object]:
    checks = []
    for fn, cid in (
        (check_command_metadata_contract, "contracts.command_metadata"),
        (check_no_duplicate_command_names, "contracts.no_duplicate_commands"),
        (check_command_help_docs_drift, "contracts.help_docs_drift"),
        (check_public_commands_docs_index, "contracts.docs_index"),
        (check_no_undocumented_help_commands, "contracts.help_documented"),
        (check_cli_canonical_paths, "contracts.cli_canonical_paths"),
        (check_no_duplicate_command_implementation_patterns, "contracts.duplicate_impl_patterns"),
        (check_command_surface_stability, "contracts.command_surface_stability"),
        (check_stable_command_no_breaking_changes, "contracts.stable_no_breaking_changes"),
    ):
        code, errors = fn(repo_root)
        checks.append({"id": cid, "status": "pass" if code == 0 else "fail", "errors": sorted(errors)})
    failed = [c for c in checks if c["status"] == "fail"]
    return {
        "schema_name": RUNTIME_CONTRACTS,
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if not failed else "error",
        "checks": [
            {"id": c["id"], "status": "ok" if c["status"] == "pass" else "error", "errors": c["errors"]}
            for c in checks
        ],
    }
