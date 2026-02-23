#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path


OWNED_PATHS = [
    "artifacts/ops/stack",
    "artifacts/evidence/stack",
    "artifacts/reports/atlasctl/ops-stack-report.json",
]


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / m).exists() for m in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--report", choices=["text", "json"], default="text")
    ns = ap.parse_args()
    root = _repo_root()

    removed: list[str] = []
    missing: list[str] = []
    for rel in OWNED_PATHS:
        p = root / rel
        if not p.exists():
            missing.append(rel)
            continue
        if not ns.dry_run:
            if p.is_dir():
                shutil.rmtree(p)
            else:
                p.unlink()
        removed.append(rel)

    if ns.report == "json":
        print(json.dumps({"schema_version": 1, "kind": "ops-stack-cleanup", "dry_run": ns.dry_run, "removed": removed, "missing": missing}, sort_keys=True))
    else:
        prefix = "dry-run: would remove" if ns.dry_run else "removed"
        print(f"{prefix}: {', '.join(removed) if removed else '(none)'}")
        if missing:
            print(f"missing: {', '.join(missing)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

