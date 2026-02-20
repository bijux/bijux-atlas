#!/usr/bin/env python3
# Purpose: enforce <=10 public entrypoints per ops area.
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
SURFACE = ROOT / "configs/ops/public-surface.json"
MAX_PER_AREA = 10


def area_of_command(cmd: str) -> str:
    name = Path(cmd).stem
    prefixes = ("stack-", "k8s-", "obs-", "load-", "datasets-", "e2e-", "report-", "fixtures-")
    for pfx in prefixes:
        if name.startswith(pfx):
            return pfx.rstrip("-")
    return "global"


def main() -> int:
    data = json.loads(SURFACE.read_text(encoding="utf-8"))
    grouped: dict[str, list[str]] = defaultdict(list)
    for cmd in data.get("ops_run_commands", []):
        grouped[area_of_command(cmd)].append(cmd)

    errors: list[str] = []
    for area, targets in sorted(grouped.items()):
        if len(targets) > MAX_PER_AREA:
            errors.append(f"{area}: {len(targets)} public entrypoints (max {MAX_PER_AREA})")

    if errors:
        print("public entrypoint cap check failed")
        for err in errors:
            print(f"- {err}")
        return 1

    print("public entrypoint cap check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
