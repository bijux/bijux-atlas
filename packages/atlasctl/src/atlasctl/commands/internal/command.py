from __future__ import annotations

import argparse
import json
import os
import sys

from ...core.context import RunContext
from ...core.exec import run
from .legacy_inventory import run_legacy_inventory
from .legacy_targets import run_legacy_targets
from .refactor_check_ids import run_refactor_check_ids

_INTERNAL_FORWARD: dict[str, str] = {
    "self-check": "self-check",
    "doctor": "doctor",
}
_INTERNAL_ITEMS: tuple[str, ...] = ("doctor", "legacy", "legacy-targets", "refactor-check-ids", "self-check")


def _forward(ctx: RunContext, *args: str) -> int:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ctx.repo_root,
        env=env,
        text=True,
    )
    return proc.returncode


def run_internal_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "internal_cmd", "")
    if not sub and bool(getattr(ns, "list", False)):
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "internal", "items": list(_INTERNAL_ITEMS)}, sort_keys=True))
        else:
            for item in _INTERNAL_ITEMS:
                print(item)
        return 0
    if sub == "legacy":
        action = getattr(ns, "legacy_cmd", "") or "inventory"
        if action == "inventory":
            return run_legacy_inventory(ctx, getattr(ns, "report", "text"))
        return 2
    if sub == "legacy-targets":
        return run_legacy_targets(ctx, getattr(ns, "report", "text"))
    if sub == "refactor-check-ids":
        code, touched = run_refactor_check_ids(ctx.repo_root, apply=bool(getattr(ns, "apply", False)))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "kind": "refactor-check-ids",
            "apply": bool(getattr(ns, "apply", False)),
            "changed_files": touched,
            "changed_count": len(touched),
            "alias_expires_on": "2026-12-31",
        }
        if bool(getattr(ns, "json", False)):
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"changed={len(touched)}")
            for rel in touched:
                print(rel)
        return code
    forwarded = _INTERNAL_FORWARD.get(sub)
    if not forwarded:
        return 2
    return _forward(ctx, forwarded, *getattr(ns, "args", []))


def configure_internal_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("internal", help="internal control-plane group (legacy inventory and diagnostics)")
    parser.add_argument("--list", action="store_true", help="list available internal commands")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    internal_sub = parser.add_subparsers(dest="internal_cmd", required=False)
    legacy = internal_sub.add_parser("legacy", help="internal legacy reports")
    legacy_sub = legacy.add_subparsers(dest="legacy_cmd", required=False)
    inventory = legacy_sub.add_parser("inventory", help="emit legacy inventory report")
    inventory.add_argument("--report", choices=["text", "json"], default="text")
    legacy.add_argument("--report", choices=["text", "json"], default="text")
    for name, help_text in (("self-check", "forward to `atlasctl self-check`"), ("doctor", "forward to `atlasctl doctor`")):
        sp = internal_sub.add_parser(name, help=help_text)
        sp.add_argument("args", nargs=argparse.REMAINDER)
    legacy_targets = internal_sub.add_parser("legacy-targets", help="list deprecated legacy targets with expiry")
    legacy_targets.add_argument("--report", choices=["text", "json"], default="text")
    refactor = internal_sub.add_parser("refactor-check-ids", help="rewrite legacy check ids to checks_* canonical ids")
    refactor.add_argument("--apply", action="store_true", help="apply edits in-place (default: dry-run)")
