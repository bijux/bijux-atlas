#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    cfg = json.loads((root / "configs/ops/cache-budget-thresholds.v1.json").read_text(encoding="utf-8"))
    budget = int(cfg.get("max_disk_bytes", 0))
    try:
        proc = subprocess.run(
            ["du", "-sk", "artifacts/e2e-store"],
            cwd=root,
            text=True,
            capture_output=True,
            check=False,
        )
        usage = 0
        if proc.returncode == 0 and proc.stdout.strip():
            usage = int(proc.stdout.split()[0]) * 1024
    except Exception:
        usage = 0
    if budget and usage > budget:
        raise SystemExit(f"cache budget exceeded: {usage} > {budget}")
    print(f"cache budget check passed: {usage}/{budget}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
