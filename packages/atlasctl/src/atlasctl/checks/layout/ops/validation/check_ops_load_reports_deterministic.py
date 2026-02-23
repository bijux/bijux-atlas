#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def main() -> int:
    src = (ROOT / "packages/atlasctl/src/atlasctl/commands/ops/load/reports/generate_markdown_summary.py").read_text(
        encoding="utf-8"
    )
    errors: list[str] = []
    if 'sorted(RES.glob("*.summary.json"))' not in src:
        errors.append("load markdown summary generator must iterate summaries in sorted order")
    forbidden = ("time.strftime(", "datetime.now(", "uuid", "random.")
    for token in forbidden:
        if token in src:
            errors.append(f"load markdown summary generator must be deterministic; found `{token}`")
    if errors:
        print("ops load report determinism check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ops load report determinism check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
