#!/usr/bin/env python3
# Purpose: ensure critical make targets are referenced in docs for discoverability.
# Inputs: docs/**/*.md.
# Outputs: non-zero when critical targets are not documented.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"

CRITICAL = [
    "docs",
    "contracts",
    "ci",
    "local",
    "local-full",
    "ops-up",
    "ops-deploy",
    "ops-smoke",
    "ops-k8s-tests",
    "ops-load-smoke",
    "ops-observability-validate",
    "ops-full",
]


def main() -> int:
    text = "\n".join(p.read_text(encoding="utf-8") for p in sorted(DOCS.rglob("*.md")))
    missing = [target for target in CRITICAL if f"`{target}`" not in text and f"make {target}" not in text]
    if missing:
        print("critical make target docs coverage failed:", file=sys.stderr)
        for target in missing:
            print(f"- missing docs reference for `{target}`", file=sys.stderr)
        return 1
    print("critical make target docs coverage passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
