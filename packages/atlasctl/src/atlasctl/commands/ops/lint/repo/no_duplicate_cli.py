#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def main() -> int:
    cli_shims = sorted((ROOT / "scripts" / "bin").glob("*")) if (ROOT / "scripts" / "bin").exists() else []
    names = {p.name for p in cli_shims if p.is_file()}
    overlap = sorted(name for name in names if name in {"atlasctl", "atlas-scripts"})
    if len(overlap) > 1:
        print("duplicate cli lint failed: overlapping script shims detected", file=sys.stderr)
        for name in overlap:
            print(f"- {name}", file=sys.stderr)
        return 1
    print("duplicate cli lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
