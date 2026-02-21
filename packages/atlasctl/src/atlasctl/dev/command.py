from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

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
    if sub == "split-module":
        return _run_split_module(ctx, ns)
    forwarded = _DEV_FORWARD.get(sub)
    if not forwarded:
        return 2
    return _forward(ctx, forwarded, *getattr(ns, "args", []))


def _run_split_module(ctx: RunContext, ns: argparse.Namespace) -> int:
    raw = str(getattr(ns, "path", "")).strip()
    if not raw:
        print("missing --path")
        return 2
    path = Path(raw)
    abs_path = path if path.is_absolute() else (ctx.repo_root / path)
    if not abs_path.exists():
        print(f"path not found: {raw}")
        return 2
    rel = abs_path.relative_to(ctx.repo_root).as_posix()
    stem = abs_path.stem
    plan = [
        f"1. Create a directory for `{stem}` responsibilities (for example `{abs_path.parent / (stem + '_parts')}`).",
        "2. Move pure domain logic into focused modules by concern (parsing, models, validation, execution).",
        "3. Keep the original command/entry wrapper thin and delegate to the new modules.",
        "4. Add or update unit tests for each extracted function before deleting old code blocks.",
        "5. Run `atlasctl policies check-py-files-per-dir --print-culprits` to verify budget recovery.",
        "6. Re-read `packages/atlasctl/docs/architecture/how-to-split-a-module.md` and align names with intent.",
    ]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "path": rel,
        "split_plan": plan,
        "recommended_doc": "packages/atlasctl/docs/architecture/how-to-split-a-module.md",
    }
    if bool(getattr(ns, "json", False)):
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"split-module plan for {rel}")
        for line in plan:
            print(f"- {line}")
        print(f"required reading: {payload['recommended_doc']}")
    return 0


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
    split = dev_sub.add_parser("split-module", help="generate a module split plan for a path")
    split.add_argument("--path", required=True)
    split.add_argument("--json", action="store_true", help="emit JSON output")
