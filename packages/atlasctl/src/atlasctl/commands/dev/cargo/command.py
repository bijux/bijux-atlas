from __future__ import annotations

import argparse

from ....core.context import RunContext
from ....core.effects.dev_cargo import DevCargoParams, run_dev_cargo


def configure_dev_cargo_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("cargo", help="dev cargo execution wrappers")
    parser.add_argument("--action", default="check")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--verbose", action="store_true")


def run_dev_cargo_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    params = DevCargoParams(action=str(getattr(ns, "action", "check")), as_json=bool(getattr(ns, "json", False)), verbose=bool(getattr(ns, "verbose", False)))
    return run_dev_cargo(ctx, params)


__all__ = ["DevCargoParams", "run_dev_cargo", "configure_dev_cargo_parser", "run_dev_cargo_command"]
