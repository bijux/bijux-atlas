from __future__ import annotations

import argparse
import json

from ..checks.registry.ssot import load_registry_entries
from ..cli.surface_registry import command_registry
from ..core.context import RunContext
from ..core.owners import load_owner_catalog


def configure_owners_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("owners", help="list and validate owner ids and command group mappings")
    owners_sub = parser.add_subparsers(dest="owners_cmd")
    owners_sub.required = True

    list_parser = owners_sub.add_parser("list", help="list owner ids from configs/meta/owners.json")
    list_parser.add_argument("--json", action="store_true", help="emit JSON output")

    validate_parser = owners_sub.add_parser("validate", help="validate check + command owners against owners.json")
    validate_parser.add_argument("--json", action="store_true", help="emit JSON output")


def run_owners_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    catalog = load_owner_catalog(ctx.repo_root)
    emit_json = bool(getattr(ns, "json", False) or ctx.output_format == "json")
    if ns.owners_cmd == "list":
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "owners-list",
            "owners": [{"id": owner} for owner in catalog.owners],
            "count": len(catalog.owners),
        }
        if emit_json:
            print(json.dumps(payload, sort_keys=True))
        else:
            for owner in catalog.owners:
                print(owner)
        return 0

    if ns.owners_cmd == "validate":
        known_owners = set(catalog.owners)
        errors: list[str] = []
        check_owners = {entry.owner for entry in load_registry_entries(ctx.repo_root)}
        for owner in sorted(check_owners):
            if owner not in known_owners:
                errors.append(f"check owner `{owner}` missing from configs/meta/owners.json")
        for spec in command_registry():
            if spec.owner not in known_owners:
                errors.append(f"command `{spec.name}` owner `{spec.owner}` missing from configs/meta/owners.json")
            group_owner = catalog.command_groups.get(spec.name.split(' ', 1)[0])
            if not group_owner:
                errors.append(f"command group `{spec.name}` missing from owners command_groups mapping")
            elif group_owner != spec.owner:
                errors.append(f"command group `{spec.name}` owner mismatch (owners.json={group_owner}, registry={spec.owner})")
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "owners-validate",
            "status": "pass" if not errors else "fail",
            "errors": errors,
        }
        if emit_json:
            print(json.dumps(payload, sort_keys=True))
        else:
            if errors:
                for err in errors:
                    print(err)
            else:
                print("owners validate: pass")
        return 0 if not errors else 1

    return 2
