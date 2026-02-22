from __future__ import annotations

import argparse

from ...core.context import RunContext
from ..internal.refactor_check_ids import run_refactor_check_ids


def run_contributing_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = str(getattr(ns, "contrib_cmd", "") or "")
    if sub == "refactor-check-ids":
        code, touched = run_refactor_check_ids(ctx.repo_root, apply=bool(getattr(ns, "apply", False)))
        if bool(getattr(ns, "json", False)):
            import json

            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok",
                        "kind": "contributing-refactor-check-ids",
                        "apply": bool(getattr(ns, "apply", False)),
                        "changed_count": len(touched),
                        "changed_files": touched,
                    },
                    sort_keys=True,
                )
            )
        else:
            print(f"changed={len(touched)}")
            for rel in touched:
                print(rel)
        return code
    return 2


def configure_contributing_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("contributing", help="contributor helpers for safe refactors")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    p_sub = parser.add_subparsers(dest="contrib_cmd", required=True)
    refactor = p_sub.add_parser("refactor-check-ids", help="rewrite legacy check ids across code/docs/goldens/policy files")
    refactor.add_argument("--apply", action="store_true", help="apply edits in place (default: dry-run)")
