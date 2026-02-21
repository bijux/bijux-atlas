from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

from .cargo.command import DevCargoParams, run_dev_cargo
from ...core.context import RunContext

_DEV_FORWARD: dict[str, str] = {
    "list": "list",
    "check": "check",
    "suite": "suite",
    "test": "test",
    "ci": "ci",
    "make": "make",
    "commands": "commands",
    "explain": "explain",
}
_DEV_ITEMS: tuple[str, ...] = (
    "audit",
    "check",
    "ci",
    "commands",
    "coverage",
    "coverage-and-slow",
    "explain",
    "fmt",
    "fmt-and-slow",
    "lint",
    "lint-and-slow",
    "list",
    "make",
    "split-module",
    "suite",
    "test",
    "test-and-slow",
    "audit-and-slow",
    "check-and-slow",
)


def _forward(ctx: RunContext, *args: str) -> int:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    forwarded_flags: list[str] = []
    if ctx.quiet:
        forwarded_flags.append("--quiet")
    if ctx.output_format == "json":
        forwarded_flags.extend(["--format", "json"])
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *forwarded_flags, *args],
        cwd=ctx.repo_root,
        env=env,
        text=True,
        check=False,
    )
    return proc.returncode


def run_dev_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "dev_cmd", "")
    slow_variant = sub.endswith("-and-slow")
    normalized_sub = sub.removesuffix("-and-slow") if slow_variant else sub
    if not sub and bool(getattr(ns, "list", False)):
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "dev", "items": list(_DEV_ITEMS)}, sort_keys=True))
        else:
            for item in _DEV_ITEMS:
                print(item)
        return 0
    if sub == "split-module":
        return _run_split_module(ctx, ns)
    if normalized_sub in {"fmt", "lint", "check", "coverage", "audit"}:
        return run_dev_cargo(
            ctx,
            DevCargoParams(
                action=normalized_sub,
                all_tests=bool(getattr(ns, "all", False) or slow_variant),
                and_checks=bool(getattr(ns, "and_checks", False)),
                explain=bool(getattr(ns, "explain", False)),
                include_slow_checks=bool(getattr(ns, "and_slow", False) or slow_variant),
                json_output=bool(getattr(ns, "json", False) or ctx.output_format == "json"),
                verbose=bool(getattr(ns, "verbose", False) or ctx.verbose),
            ),
        )
    if normalized_sub == "test":
        args = list(getattr(ns, "args", []))
        if args:
            return _forward(ctx, "test", *args)
        return run_dev_cargo(
            ctx,
            DevCargoParams(
                action="test",
                all_tests=bool(getattr(ns, "all", False) or slow_variant),
                contracts_tests=bool(getattr(ns, "contracts", False)),
                and_checks=bool(getattr(ns, "and_checks", False)),
                explain=bool(getattr(ns, "explain", False)),
                include_slow_checks=bool(getattr(ns, "and_slow", False) or slow_variant),
                json_output=bool(getattr(ns, "json", False) or ctx.output_format == "json"),
                verbose=bool(getattr(ns, "verbose", False) or ctx.verbose),
            ),
        )
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
    parser.add_argument("--list", action="store_true", help="list available dev commands")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    parser.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    dev_sub = parser.add_subparsers(dest="dev_cmd", required=False)
    for name, help_text in (
        ("list", "forward to `atlasctl list ...`"),
        ("suite", "forward to `atlasctl suite ...`"),
        ("ci", "forward to `atlasctl ci ...`"),
        ("make", "forward to `atlasctl make ...`"),
        ("commands", "forward to `atlasctl commands ...`"),
        ("explain", "forward to `atlasctl explain ...`"),
    ):
        sp = dev_sub.add_parser(name, help=help_text)
        sp.add_argument("args", nargs=argparse.REMAINDER)
    fmt = dev_sub.add_parser("fmt", help="run canonical cargo fmt lane")
    fmt.add_argument("--all", action="store_true", help="run full fmt variant")
    fmt.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    fmt.add_argument("--explain", action="store_true", help="print planned steps without executing")
    fmt.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    dev_sub.add_parser("fmt-and-slow", help="run fmt full variant including slow repo checks")
    lint = dev_sub.add_parser("lint", help="run canonical cargo lint lane")
    lint.add_argument("--all", action="store_true", help="run full lint variant")
    lint.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    lint.add_argument("--explain", action="store_true", help="print planned steps without executing")
    lint.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    dev_sub.add_parser("lint-and-slow", help="run lint full variant including slow repo checks")
    check = dev_sub.add_parser("check", help="run canonical cargo check lane")
    check.add_argument("--all", action="store_true", help="run full check variant")
    check.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    check.add_argument("--explain", action="store_true", help="print planned steps without executing")
    check.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    check.add_argument("args", nargs=argparse.REMAINDER)
    dev_sub.add_parser("check-and-slow", help="run check full variant including slow repo checks")
    test = dev_sub.add_parser("test", help="run canonical cargo test lane")
    test.add_argument("--all", action="store_true", help="run ignored tests too")
    test.add_argument("--contracts", action="store_true", help="run contracts-only tests")
    test.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    test.add_argument("--explain", action="store_true", help="print planned steps without executing")
    test.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    test.add_argument("args", nargs=argparse.REMAINDER)
    dev_sub.add_parser("test-and-slow", help="run test full variant including slow repo checks")
    coverage = dev_sub.add_parser("coverage", help="run canonical cargo coverage lane")
    coverage.add_argument("--all", action="store_true", help="run full coverage variant")
    coverage.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    coverage.add_argument("--explain", action="store_true", help="print planned steps without executing")
    coverage.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    dev_sub.add_parser("coverage-and-slow", help="run coverage full variant including slow repo checks")
    audit = dev_sub.add_parser("audit", help="run canonical cargo audit lane")
    audit.add_argument("--all", action="store_true", help="run full audit variant")
    audit.add_argument("--and-checks", action="store_true", help="append repo checks to the fast variant")
    audit.add_argument("--explain", action="store_true", help="print planned steps without executing")
    audit.add_argument("--and-slow", action="store_true", help="include slow repo checks when running full variant")
    dev_sub.add_parser("audit-and-slow", help="run audit full variant including slow repo checks")
    split = dev_sub.add_parser("split-module", help="generate a module split plan for a path")
    split.add_argument("--path", required=True)
    split.add_argument("--json", action="store_true", help="emit JSON output")
