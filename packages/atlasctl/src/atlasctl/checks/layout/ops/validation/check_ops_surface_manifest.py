#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
MANIFEST = ROOT / "configs/ops/ops-surface-manifest.json"
SURFACE = ROOT / "ops/_meta/surface.json"
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
    if not SURFACE.exists():
        errs.append("missing ops/_meta/surface.json")
    else:
        try:
            surface = json.loads(SURFACE.read_text(encoding="utf-8"))
        except Exception as exc:
            errs.append(f"invalid ops/_meta/surface.json: {exc}")
            surface = {}
        actions = surface.get("actions", []) if isinstance(surface, dict) else []
        if not isinstance(actions, list) or not actions:
            errs.append("ops/_meta/surface.json: actions must be non-empty list")
        else:
            for row in actions[:5]:
                if not isinstance(row, dict):
                    errs.append("ops/_meta/surface.json: action rows must be objects")
                    break
            for row in actions:
                if not isinstance(row, dict):
                    continue
                aid = str(row.get("id", "")).strip()
                cmd = row.get("command", [])
                if not aid.startswith("ops."):
                    errs.append(f"invalid action id `{aid}` in ops/_meta/surface.json")
                if not isinstance(cmd, list) or cmd[:2] != ["atlasctl", "ops"]:
                    errs.append(f"action `{aid}` must map to atlasctl ops command")
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
