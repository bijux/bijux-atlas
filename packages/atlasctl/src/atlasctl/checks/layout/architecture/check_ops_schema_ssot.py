#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
PKG_SRC = ROOT / "packages/atlasctl/src"
if str(PKG_SRC) not in sys.path:
    sys.path.insert(0, str(PKG_SRC))

from atlasctl.ops.contracts import ops_schema_contracts


def _expected_id(path: str) -> str:
    rel = path.removeprefix("ops/_schemas/")
    if rel.startswith("e2e-"):
        rel = "e2e/" + rel[len("e2e-") :]
    if rel.endswith(".schema.json"):
        rel = rel[: -len(".schema.json")]
    elif rel.endswith(".json"):
        rel = rel[: -len(".json")]
    p = rel.replace("/", ".")
    return f"ops.{p}"


def main() -> int:
    errs: list[str] = []
    contracts = ops_schema_contracts(ROOT)
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    for row in contracts:
        if row.id in seen_ids:
            errs.append(f"duplicate schema contract id: {row.id}")
        seen_ids.add(row.id)
        if row.path in seen_paths:
            errs.append(f"duplicate schema contract path: {row.path}")
        seen_paths.add(row.path)
        path = ROOT / row.path
        if not path.exists():
            errs.append(f"missing schema file: {row.path}")
        exp = _expected_id(row.path)
        if row.id != exp:
            errs.append(f"{row.path}: contract id `{row.id}` must match filename-derived id `{exp}`")
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errs.append(f"{row.path}: invalid json ({exc})")
    actual = {
        p.relative_to(ROOT).as_posix()
        for p in (ROOT / "ops/_schemas").rglob("*.json")
        if p.name != ".gitkeep"
    }
    missing = sorted(actual - seen_paths)
    extra = sorted(seen_paths - actual)
    for path in missing:
        errs.append(f"schema missing from configs/ops/schema-contracts.json: {path}")
    for path in extra:
        errs.append(f"stale schema contract path: {path}")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops schema SSOT OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
