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
    nightly = [p for p in profiles if p.get("suite") == "nightly"]
    if len(nightly) < 3:
        raise SystemExit("nightly install matrix must include at least three profiles")
    required = {"perf", "multi-registry", "ingress"}
    names = {p.get("name") for p in nightly}
    missing = sorted(required - names)
    if missing:
        raise SystemExit(f"nightly install matrix missing required profiles: {missing}")
    print("install matrix full contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
