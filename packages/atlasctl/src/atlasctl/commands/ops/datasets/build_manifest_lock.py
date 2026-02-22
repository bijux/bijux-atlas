#!/usr/bin/env python3
from __future__ import annotations

import hashlib
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
    manifest = json.loads((root / "ops/datasets/manifest.json").read_text(encoding="utf-8"))
    entries: list[dict[str, object]] = []
    for ds in manifest.get("datasets", []):
        if not isinstance(ds, dict):
            continue
        checksums: dict[str, str | None] = {}
        paths = ds.get("paths") if isinstance(ds.get("paths"), dict) else {}
        for key, rel in sorted(paths.items()):
            p = root / str(rel)
            checksums[str(key)] = hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
        entries.append({"name": ds.get("name"), "id": ds.get("id"), "checksums": checksums})
    out = {"schema_version": 1, "entries": entries}
    target = root / "ops/datasets/manifest.lock"
    target.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
    print("wrote ops/datasets/manifest.lock")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
