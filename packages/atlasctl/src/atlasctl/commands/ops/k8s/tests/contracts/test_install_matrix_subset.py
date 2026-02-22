#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    matrix = json.loads((root / "ops/k8s/install-matrix.json").read_text(encoding="utf-8"))
    profiles = matrix.get("profiles", [])
    subset = [p for p in profiles if p.get("suite") == "install-gate"]
    if len(subset) == 0:
        raise SystemExit("install matrix subset missing: expected at least one install-gate profile")
    if len(subset) > 2:
        raise SystemExit("install matrix subset too large for local runs (max 2 install-gate profiles)")
    names = {p.get("name") for p in subset}
    if "local" not in names:
        raise SystemExit("install matrix subset must include local profile")
    print("install matrix subset contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
