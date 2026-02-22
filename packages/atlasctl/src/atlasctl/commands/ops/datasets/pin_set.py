#!/usr/bin/env python3
from __future__ import annotations

import os
import shlex
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
    out_dir = root / "artifacts/ops/cache"
    out_dir.mkdir(parents=True, exist_ok=True)
    datasets = os.environ.get("DATASETS") or os.environ.get("ATLAS_PINNED_DATASETS", "")
    if not datasets:
        print("usage: DATASETS=release/species/assembly[,..] make ops-cache-pin-set", file=sys.stderr)
        return 2
    env_file = out_dir / "pins.env"
    env_file.write_text(f"ATLAS_PINNED_DATASETS={shlex.quote(datasets)}\n", encoding="utf-8")
    print(f"wrote {env_file}")
    print(f"export ATLAS_PINNED_DATASETS='{datasets}'")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
