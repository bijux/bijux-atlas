#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
REG = ROOT / "configs/ops/manifests-registry.json"


def main() -> int:
    payload = json.loads(REG.read_text(encoding="utf-8"))
    rows = payload.get("items", [])
    errs: list[str] = []
    seen: set[str] = set()
    for row in rows:
        path = str(row.get("path", "")).strip()
        if not path:
            errs.append("manifest registry row missing path")
            continue
        if path in seen:
            errs.append(f"duplicate manifest registry path: {path}")
        seen.add(path)
        if not (ROOT / path).exists():
            errs.append(f"manifest registry path missing on disk: {path}")
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.joinpath("ops").rglob("manifest.json")}
    for path in sorted(actual - seen):
        errs.append(f"ops manifest missing from registry: {path}")
    for path in sorted(seen - actual):
        errs.append(f"stale registry manifest path: {path}")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops manifest registry SSOT OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
