from __future__ import annotations

import argparse
import os
import subprocess
import sys

from ..core.context import RunContext

_DEV_FORWARD: dict[str, str] = {
    "list": "list",
    "check": "check",
    "suite": "suite",
    "test": "test",
    "commands": "commands",
    "explain": "explain",
}


def _forward(ctx: RunContext, *args: str) -> int:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ctx.repo_root,
        env=env,
        text=True,
        check=False,
    )
    return proc.returncode


def run_dev_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "dev_cmd", "")
    forwarded = _DEV_FORWARD.get(sub)
    if not forwarded:
        return 2
    return _forward(ctx, forwarded, *getattr(ns, "args", []))


def configure_dev_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("dev", help="dev control-plane group (checks, suites, tests, listing)")
    dev_sub = parser.add_subparsers(dest="dev_cmd", required=True)
    for name, help_text in (
        ("list", "forward to `atlasctl list ...`"),
        ("check", "forward to `atlasctl check ...`"),
        ("suite", "forward to `atlasctl suite ...`"),
        ("test", "forward to `atlasctl test ...`"),
        ("commands", "forward to `atlasctl commands ...`"),
        ("explain", "forward to `atlasctl explain ...`"),
    ):
        sp = dev_sub.add_parser(name, help=help_text)
        sp.add_argument("args", nargs=argparse.REMAINDER)
