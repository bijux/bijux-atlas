from __future__ import annotations

import argparse
import json

from ...core.context import RunContext


def configure_packages_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("packages", help="package domain inventory commands")
    parser.add_argument("--json", action="store_true", help="emit JSON output")


def run_packages_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "group": "packages",
        "run_id": ctx.run_id,
        "next": "add package lifecycle subcommands under atlasctl commands/packages/",
    }
    if bool(getattr(ns, "json", False) or ctx.output_format == "json"):
        print(json.dumps(payload, sort_keys=True))
    else:
        print("packages: ok")
    return 0

