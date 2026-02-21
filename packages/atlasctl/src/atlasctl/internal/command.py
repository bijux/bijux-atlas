from __future__ import annotations

import argparse
import os
import subprocess
import sys

from ..core.context import RunContext

_INTERNAL_FORWARD: dict[str, str] = {
    "legacy": "legacy",
    "compat": "compat",
    "self-check": "self-check",
    "doctor": "doctor",
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


def run_internal_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "internal_cmd", "")
    forwarded = _INTERNAL_FORWARD.get(sub)
    if not forwarded:
        return 2
    return _forward(ctx, forwarded, *getattr(ns, "args", []))


def configure_internal_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("internal", help="internal control-plane group (legacy/compat/diagnostics)")
    internal_sub = parser.add_subparsers(dest="internal_cmd", required=True)
    for name, help_text in (
        ("legacy", "forward to `atlasctl legacy ...`"),
        ("compat", "forward to `atlasctl compat ...`"),
        ("self-check", "forward to `atlasctl self-check`"),
        ("doctor", "forward to `atlasctl doctor`"),
    ):
        sp = internal_sub.add_parser(name, help=help_text)
        sp.add_argument("args", nargs=argparse.REMAINDER)
