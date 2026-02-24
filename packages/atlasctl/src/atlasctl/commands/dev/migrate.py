from __future__ import annotations

import argparse
import json

from ...checks.registry import registry_delta
from ...core.context import RunContext


def run_migrate_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.migrate_cmd != "checks-registry":
        return 2
    delta = registry_delta(ctx.repo_root)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if not delta["unregistered_implementations"] and not delta["orphan_registry_entries"] else "error",
        "kind": "migrate-checks-registry",
        **delta,
    }
    if ctx.output_format == "json" or getattr(ns, "json", False):
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"unregistered_implementations={len(delta['unregistered_implementations'])}")
        for item in delta["unregistered_implementations"]:
            print(f"- {item}")
        print(f"orphan_registry_entries={len(delta['orphan_registry_entries'])}")
        for item in delta["orphan_registry_entries"]:
            print(f"- {item}")
    return 0 if payload["status"] == "ok" else 1


def configure_migrate_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("migrate", help="migration helper commands")
    p.add_argument("--json", action="store_true", help="emit JSON output")
    s = p.add_subparsers(dest="migrate_cmd", required=True)
    s.add_parser("checks-registry", help="show registry/implementation migration delta")

