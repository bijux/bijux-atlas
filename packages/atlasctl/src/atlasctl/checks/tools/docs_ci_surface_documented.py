#!/usr/bin/env python3
# Purpose: ensure DEV/CI stable command surface docs stay aligned with parser/help.
from __future__ import annotations

import sys
from pathlib import Path

from ....core.process import run_command

ROOT = Path(__file__).resolve().parents[7]
DOC = ROOT / "packages" / "atlasctl" / "docs" / "control-plane" / "dev-ci-surface.md"
PUBLIC_API_CANDIDATES = (
    ROOT / "packages" / "atlasctl" / "docs" / "PUBLIC_API.md",
    ROOT / "packages" / "atlasctl" / "docs" / "public-api.md",
    ROOT / "docs" / "PUBLIC_API.md",
    ROOT / "docs" / "public-api.md",
)
REQUIRED = (
    "atlasctl dev ci run",
    "atlasctl dev fmt",
    "atlasctl dev lint",
    "atlasctl dev test",
    "atlasctl dev coverage",
    "atlasctl dev audit",
)


def _help(cmd: list[str]) -> str:
    proc = run_command(cmd, cwd=ROOT)
    return (proc.stdout or "") + (proc.stderr or "")


def main() -> int:
    doc_text = DOC.read_text(encoding="utf-8")
    public_api = next((path for path in PUBLIC_API_CANDIDATES if path.exists()), None)
    if public_api is None:
        print("ci surface documented check failed", file=sys.stderr)
        print("- missing docs public api file (expected docs/PUBLIC_API.md or docs/public-api.md)", file=sys.stderr)
        return 1
    public_text = public_api.read_text(encoding="utf-8")
    errors: list[str] = []
    for command in REQUIRED:
        if command not in doc_text:
            errors.append(f"missing from {DOC.relative_to(ROOT)}: `{command}`")
        if command not in public_text:
            errors.append(f"missing from {public_api.relative_to(ROOT)}: `{command}`")

    dev_help = _help(["./bin/atlasctl", "dev", "--help"])
    ci_run_help = run_command(
        ["./bin/atlasctl", "dev", "ci", "run", "--help"],
        cwd=ROOT,
    )
    for command in ("fmt", "lint", "test", "coverage", "audit"):
        if command not in dev_help:
            errors.append(f"`atlasctl dev --help` missing subcommand `{command}`")
    if ci_run_help.code != 0:
        errors.append("`atlasctl dev ci run --help` failed")

    if errors:
        print("ci surface documented check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("ci surface documented check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
