#!/usr/bin/env python3
# Purpose: ban direct cargo test/fmt/clippy and internal make target invocations in workflows.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))

CARGO_RE = re.compile(r"\bcargo\s+(?:test|fmt|clippy)\b")
MAKE_INTERNAL_RE = re.compile(r"\bmake\s+[^#\n]*\binternal/[A-Za-z0-9_./-]+")


def main() -> int:
    errors: list[str] = []
    for workflow in WORKFLOWS:
        text = workflow.read_text(encoding="utf-8")
        for lineno, line in enumerate(text.splitlines(), start=1):
            if CARGO_RE.search(line):
                errors.append(
                    f"{workflow.relative_to(ROOT)}:{lineno}: forbidden direct cargo invocation in workflow line"
                )
            if MAKE_INTERNAL_RE.search(line):
                errors.append(
                    f"{workflow.relative_to(ROOT)}:{lineno}: forbidden internal make target invocation in workflow line"
                )

    if errors:
        print("workflow command ban check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("workflow command ban check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
