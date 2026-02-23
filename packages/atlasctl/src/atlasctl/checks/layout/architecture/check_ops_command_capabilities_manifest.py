#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
PKG_SRC = ROOT / "packages/atlasctl/src"
if str(PKG_SRC) not in sys.path:
    sys.path.insert(0, str(PKG_SRC))

from atlasctl.commands.ops.runtime_modules.actions_inventory import list_ops_actions

MANIFEST = ROOT / "configs/ops/command-capabilities.json"


def main() -> int:
    payload = json.loads(MANIFEST.read_text(encoding="utf-8"))
    rows = payload.get("items", [])
    if not isinstance(rows, list):
        print("invalid command capabilities manifest: `items` must be a list")
        return 1
    by_action: dict[str, dict[str, object]] = {}
    errs: list[str] = []
    for row in rows:
        if not isinstance(row, dict):
            errs.append("manifest row must be object")
            continue
        action = str(row.get("action", "")).strip()
        if not action:
            errs.append("manifest row missing action")
            continue
        by_action[action] = row
        for key in ("tools", "network", "allow_network_required", "profiles"):
            if key not in row:
                errs.append(f"{action}: missing `{key}`")
        if row.get("network") and not row.get("allow_network_required"):
            errs.append(f"{action}: network=true requires allow_network_required=true")
    inventory = list_ops_actions()
    missing = sorted(set(inventory) - set(by_action))
    extra = sorted(set(by_action) - set(inventory))
    for action in missing:
        errs.append(f"missing capability manifest entry: {action}")
    for action in extra:
        errs.append(f"stale capability manifest entry: {action}")
    if errs:
        print("\n".join(errs))
        return 1
    print("ops command capability manifest OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
