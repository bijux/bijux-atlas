#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
MANIFEST = ROOT / "configs/ops/external-tools-allowlist.json"
EXPECTED_AREAS = {"stack", "deploy", "k8s", "obs", "load", "e2e", "datasets", "pins", "reports"}


def main() -> int:
    payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
    errs: list[str] = []
    if int(payload.get("schema_version", 0) or 0) != 1:
        errs.append("schema_version must be 1")
    areas = payload.get("areas")
    if not isinstance(areas, dict):
        errs.append("areas must be an object")
        areas = {}
    missing = sorted(EXPECTED_AREAS - set(str(k) for k in areas.keys()))
    if missing:
        errs.append(f"missing declared ops tool areas: {missing}")
    for area, tools in sorted(areas.items()):
        if not isinstance(tools, list) or not all(isinstance(t, str) and t for t in tools):
            errs.append(f"area {area} tools must be non-empty string list")
    if errs:
        print("ops external tools manifest failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("ops external tools manifest passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
