#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
PUBLIC = ROOT / "configs/ops/public-surface.json"
OUT = ROOT / "ops/inventory/surfaces.json"
ATLASCTL_SRC = ROOT / "packages" / "atlasctl" / "src"
if str(ATLASCTL_SRC) not in sys.path:
    sys.path.insert(0, str(ATLASCTL_SRC))


def _action_registry_from_surface() -> list[dict[str, object]]:
    if OUT.exists():
        payload = json.loads(OUT.read_text(encoding="utf-8"))
        rows = payload.get("actions", [])
        if isinstance(rows, list):
            return [row for row in rows if isinstance(row, dict)]
    return []


def main() -> int:
    data = json.loads(PUBLIC.read_text(encoding="utf-8"))
    make_targets = data.get("make_targets", [])
    ops_targets = sorted(
        {
            t
            for t in make_targets
            if isinstance(t, str) and t.startswith("ops-")
        }
        | {"ops-help", "ops-surface", "ops-layout-lint", "ops-e2e-validate"}
    )
    actions = _action_registry_from_surface()
    payload = {
        "schema_version": 2,
        "entrypoints": ops_targets,
        "atlasctl_commands": [" ".join(row["command"]) for row in actions],
        "actions": actions,
    }
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(OUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
