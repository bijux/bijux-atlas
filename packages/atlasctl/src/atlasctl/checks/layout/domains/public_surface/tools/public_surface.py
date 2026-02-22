#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    data = json.loads(SURFACE.read_text(encoding="utf-8"))
    for key in ("make_targets", "ops_run_commands"):
        if key not in data or not isinstance(data[key], list):
            raise SystemExit(f"public-surface missing list: {key}")
    return data
