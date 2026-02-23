#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
PKG_SRC = ROOT / "packages/atlasctl/src"
if str(PKG_SRC) not in sys.path:
    sys.path.insert(0, str(PKG_SRC))

CONTRACTS = ROOT / "configs/ops/schema-contracts.json"


def _expected_id(path: str) -> str:
    rel = path.removeprefix("ops/schema/")
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
    payload = json.loads(CONTRACTS.read_text(encoding="utf-8"))
    rows = payload.get("contracts", [])
    if not isinstance(rows, list):
        print("invalid schema-contracts manifest: `contracts` must be a list")
        return 1
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            errs.append("schema-contracts row must be object")
            continue
        cid = str(row.get("id", "")).strip()
        cpath = str(row.get("path", "")).strip()
        if not cid or not cpath:
            errs.append("schema-contracts row must include non-empty id/path")
            continue
        if cid in seen_ids:
            errs.append(f"duplicate schema contract id: {cid}")
        seen_ids.add(cid)
        if cpath in seen_paths:
            errs.append(f"duplicate schema contract path: {cpath}")
        seen_paths.add(cpath)
        path = ROOT / cpath
        if not path.exists():
            errs.append(f"missing schema file: {cpath}")
        exp = _expected_id(cpath)
        if cid != exp:
            errs.append(f"{cpath}: contract id `{cid}` must match filename-derived id `{exp}`")
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except Exception as exc:
            errs.append(f"{cpath}: invalid json ({exc})")
    actual = {
        p.relative_to(ROOT).as_posix()
        for p in (ROOT / "ops/schema").rglob("*.json")
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
