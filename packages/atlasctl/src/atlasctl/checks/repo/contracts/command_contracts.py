from __future__ import annotations

import json
from pathlib import Path

from ....cli.registry import command_registry


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
        "schema_name": "atlasctl.runtime_contracts.v1",
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if not failed else "error",
        "checks": [
            {"id": c["id"], "status": "ok" if c["status"] == "pass" else "error", "errors": c["errors"]}
            for c in checks
        ],
    }
