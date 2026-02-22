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
    thresholds = json.loads((root / "configs/ops/cache-budget-thresholds.v1.json").read_text(encoding="utf-8"))
    status_file = root / "artifacts/ops/cache-status.json"
    if not status_file.exists():
        print(f"missing cache status artifact: {status_file}", file=sys.stderr)
        return 1
    status = json.loads(status_file.read_text(encoding="utf-8"))
    min_hit_ratio = float(thresholds.get("min_hit_ratio", 0.0))
    max_disk_bytes = int(thresholds.get("max_disk_bytes", 0))
    hit_ratio = float(status.get("cache_hit_ratio", 0.0))
    disk_bytes = int(status.get("cache_disk_bytes", 0))
    if hit_ratio < min_hit_ratio:
        raise SystemExit(f"cache hit ratio below threshold: {hit_ratio:.4f} < {min_hit_ratio:.4f}")
    if max_disk_bytes > 0 and disk_bytes > max_disk_bytes:
        raise SystemExit(f"cache disk bytes above threshold: {disk_bytes} > {max_disk_bytes}")
    print(f"cache threshold check passed: hit_ratio={hit_ratio:.4f}, disk_bytes={disk_bytes}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
