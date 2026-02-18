#!/usr/bin/env python3
# Purpose: forbid docs references to removed legacy root ops paths.
# Inputs: docs/**/*.md
# Outputs: non-zero when legacy path references are found.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
PATTERNS = [
    re.compile(r"(^|[`\s])\.?/charts/"),
    re.compile(r"(^|[`\s])\.?/e2e/"),
    re.compile(r"(^|[`\s])\.?/load/"),
    re.compile(r"(^|[`\s])\.?/observability/"),
    re.compile(r"(^|[`\s])\.?/datasets/"),
    re.compile(r"(^|[`\s])\.?/fixtures/"),
]
EXCEPTIONS = {"docs/operations/migration-note.md"}


def main() -> int:
    violations: list[str] = []
    for path in sorted(DOCS.rglob("*.md")):
        rel = path.relative_to(ROOT)
        if str(rel) in EXCEPTIONS:
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            for pat in PATTERNS:
                if pat.search(line):
                    violations.append(f"{rel}:{lineno}: legacy root path reference")
                    break
    if violations:
        print("legacy root path docs check failed:", file=sys.stderr)
        for item in violations[:200]:
            print(f"- {item}", file=sys.stderr)
        return 1
    print("legacy root path docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
