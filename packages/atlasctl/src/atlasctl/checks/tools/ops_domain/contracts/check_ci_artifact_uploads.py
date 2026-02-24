#!/usr/bin/env python3
# Purpose: ensure CI workflows always upload artifacts for reports/logs on failure.
from __future__ import annotations

import re
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists() and (base / ".github").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
WORKFLOWS = sorted((ROOT / ".github" / "workflows").glob("*.yml"))
CI_TRIGGER_RE = re.compile(r"(make\s+ci\b|\.\/bin\/atlasctl\s+ci\s+run\b)")


def main() -> int:
    errors: list[str] = []
    for workflow in WORKFLOWS:
        text = workflow.read_text(encoding="utf-8")
        if not CI_TRIGGER_RE.search(text):
            continue
        has_upload = "uses: actions/upload-artifact@v4" in text
        has_always = re.search(r"if:\s*always\(\)", text) is not None
        if not has_upload or not has_always:
            errors.append(
                f"{workflow.relative_to(ROOT)}: workflows running CI must upload artifacts with `if: always()`"
            )
    if errors:
        print("ci artifact upload check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ci artifact upload check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
