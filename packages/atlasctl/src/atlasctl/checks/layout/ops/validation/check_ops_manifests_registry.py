#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
REG = ROOT / "configs" / "ops" / "manifests-registry.json"


def _schema_exists(schema_ref: str, schema_path: str) -> bool:
    if schema_ref:
        return (ROOT / "packages" / "atlasctl" / "src" / "atlasctl" / "contracts" / "schema" / "schemas" / f"{schema_ref}.schema.json").exists()
    if schema_path:
        return (ROOT / schema_path).exists()
    return False


def main() -> int:
    payload = json.loads(REG.read_text(encoding="utf-8"))
    rows = payload.get("entries", []) if isinstance(payload, dict) else []
    errors: list[str] = []
    schema_samples: dict[str, int] = {}
    seen_paths: dict[str, int] = {}
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            errors.append(f"entries[{i}] must be object")
            continue
        rel = str(row.get("path", "")).strip()
        schema_ref = str(row.get("schema_ref", "")).strip()
        schema_path = str(row.get("schema_path", "")).strip()
        if not rel:
            errors.append(f"entries[{i}]: path is required")
            continue
        seen_paths[rel] = seen_paths.get(rel, 0) + 1
        if not (ROOT / rel).exists():
            errors.append(f"missing manifest: {rel}")
        if not _schema_exists(schema_ref, schema_path):
            errors.append(f"{rel}: missing schema ({schema_ref or schema_path or 'unset'})")
        key = schema_ref or schema_path
        if key:
            schema_samples[key] = schema_samples.get(key, 0) + 1
    for rel, count in sorted(seen_paths.items()):
        if count != 1:
            errors.append(f"{rel}: manifest must be referenced exactly once in manifests-registry.json")
    if not any(r for r in rows if isinstance(r, dict) and str(r.get("path", "")).strip() == "ops/e2e/manifests/smoke.manifest.json"):
        errors.append("ops/e2e/manifests/smoke.manifest.json must be first-class in manifests-registry.json")
    if errors:
        print("ops manifests registry check failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("ops manifests registry check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
