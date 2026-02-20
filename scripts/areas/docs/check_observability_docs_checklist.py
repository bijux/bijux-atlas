#!/usr/bin/env python3
# owner: docs-governance
# purpose: enforce observability docs checklist and required section headings.
# stability: public
# called-by: make docs, make docs-lint-names
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHECKLIST = ROOT / "docs/_lint/observability-docs.md"
OBS_DIR = ROOT / "docs/operations/observability"

REQUIRED_PAGES = {
    "INDEX.md",
    "acceptance-gates.md",
    "alerts.md",
    "dashboard.md",
    "profiles.md",
    "slo.md",
    "tracing.md",
    "compatibility.md",
}

REQUIRED_HEADINGS = ["## What", "## Why", "## Contracts", "## Failure modes", "## How to verify"]


def main() -> int:
    errors: list[str] = []
    if not CHECKLIST.exists():
        errors.append("missing docs/_lint/observability-docs.md")
    else:
        checklist = CHECKLIST.read_text(encoding="utf-8")
        for page in sorted(REQUIRED_PAGES):
            needle = f"- [x] `{page}`"
            if needle not in checklist:
                errors.append(f"checklist missing completed item: {needle}")

    for page in sorted(REQUIRED_PAGES):
        path = OBS_DIR / page
        if not path.exists():
            errors.append(f"missing observability page: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for heading in REQUIRED_HEADINGS:
            if heading not in text:
                errors.append(f"{path.relative_to(ROOT)} missing heading: {heading}")

    # Ensure durable heading intent for drill instructions.
    alerts = (OBS_DIR / "alerts.md").read_text(encoding="utf-8") if (OBS_DIR / "alerts.md").exists() else ""
    if "## Run drills" not in alerts:
        errors.append("docs/operations/observability/alerts.md missing heading: ## Run drills")

    if errors:
      print("observability docs checklist failed:", file=sys.stderr)
      for err in errors:
          print(f"- {err}", file=sys.stderr)
      return 1
    print("observability docs checklist passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
