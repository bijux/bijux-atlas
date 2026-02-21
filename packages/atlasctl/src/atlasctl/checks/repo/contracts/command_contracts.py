from __future__ import annotations

import json
from pathlib import Path

from ....cli.registry import command_registry
from ....contracts.ids import RUNTIME_CONTRACTS

_PUBLIC_UNSTABLE_ALLOWLIST = {"compat"}


def check_command_metadata_contract(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for spec in command_registry():
        if not spec.touches:
            errors.append(f"{spec.name}: missing touches metadata")
        if spec.tools is None:
            errors.append(f"{spec.name}: missing tools metadata")
        if not spec.owner:
            errors.append(f"{spec.name}: missing owner metadata")
        if not spec.doc_link:
            errors.append(f"{spec.name}: missing doc_link metadata")
    if errors:
        return 1, errors
    return 0, []


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
    # Legacy/compat aliases are represented as distinct commands and should be minimized.
    names = [spec.name for spec in command_registry()]
    if len(names) != len(set(names)):
        return 1, ["alias/name budget exceeded: duplicate command identifiers present"]
    return 0, []


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
        if spec.name not in listed:
            errors.append(f"{spec.name}: missing from docs/_generated/cli.md")
    return (0 if not errors else 1), errors


def runtime_contracts_payload(repo_root: Path) -> dict[str, object]:
    checks = []
    for fn, cid in (
        (check_command_metadata_contract, "contracts.command_metadata"),
        (check_no_duplicate_command_names, "contracts.no_duplicate_commands"),
        (check_command_help_docs_drift, "contracts.help_docs_drift"),
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
