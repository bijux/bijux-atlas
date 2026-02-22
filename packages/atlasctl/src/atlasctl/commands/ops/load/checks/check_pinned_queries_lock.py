#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

from atlasctl.checks.domains.ops.ops_checks import check_ops_load_pinned_queries_lock_native


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if all((base / part).exists() for part in ("makefiles", "packages", "configs", "ops")):
            return base
    raise RuntimeError("unable to resolve repository root")


def main() -> int:
    repo_root = _repo_root()
    code, rows = check_ops_load_pinned_queries_lock_native(repo_root)
    stream = sys.stderr if code else sys.stdout
    for row in rows:
        print(row, file=stream)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
