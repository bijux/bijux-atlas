#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
MANIFEST = ROOT / "configs/ops/ops-surface-manifest.json"
OPS_COMMAND = "packages/atlasctl/src/atlasctl/commands/ops/command.py"


def main() -> int:
    payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
    areas = payload.get("areas", {})
    errs: list[str] = []
    if int(payload.get("schema_version", 0) or 0) != 1:
        errs.append("schema_version must be 1")
    if not isinstance(areas, dict):
        errs.append("areas must be object")
        areas = {}
    for area, row in sorted(areas.items()):
        if not isinstance(row, dict):
            errs.append(f"{area}: entry must be object")
            continue
        entry = str(row.get("entrypoint", "")).strip()
        runtime = str(row.get("runtime", "")).strip() if row.get("runtime") else ""
        if not entry:
            errs.append(f"{area}: missing entrypoint")
        if entry and not (ROOT / entry).exists():
            errs.append(f"{area}: missing entrypoint file {entry}")
        if runtime and not (ROOT / runtime).exists():
            errs.append(f"{area}: missing runtime file {runtime}")
        if entry != OPS_COMMAND:
            errs.append(f"{area}: public ops area entrypoint must be {OPS_COMMAND}")
    if errs:
        print("ops surface manifest failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("ops surface manifest passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
