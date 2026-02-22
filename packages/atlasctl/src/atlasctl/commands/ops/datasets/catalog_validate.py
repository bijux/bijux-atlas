#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    catalog = root / "artifacts/e2e-datasets/catalog.json"
    if not catalog.exists():
        print("missing artifacts/e2e-datasets/catalog.json (run make ops-publish first)", file=sys.stderr)
        return 1
    data = json.loads(catalog.read_text(encoding="utf-8"))
    if "datasets" not in data or not isinstance(data["datasets"], list):
        print("catalog schema invalid: missing datasets[]", file=sys.stderr)
        return 1
    ids: list[str] = []
    for entry in data["datasets"]:
        ds = entry.get("dataset", {}) if isinstance(entry, dict) else {}
        ids.append(f"{ds.get('release')}/{ds.get('species')}/{ds.get('assembly')}")
    if ids != sorted(ids):
        print("catalog deterministic merge check failed: dataset ids not sorted", file=sys.stderr)
        return 1
    print("catalog validation passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
