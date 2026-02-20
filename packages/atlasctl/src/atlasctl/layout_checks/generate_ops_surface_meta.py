#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
PUBLIC = ROOT / "configs/ops/public-surface.json"
OUT = ROOT / "ops/_meta/surface.json"


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
    payload = {"schema_version": 1, "entrypoints": ops_targets}
    OUT.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(OUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
