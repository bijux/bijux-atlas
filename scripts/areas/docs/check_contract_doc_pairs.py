#!/usr/bin/env python3
# Purpose: enforce contract registry coverage by generated or hand-written docs.
# Inputs: docs/contracts/*.json, docs/contracts/*.md, docs/_generated/contracts/*.md.
# Outputs: non-zero exit when a contract JSON lacks corresponding documentation.
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTRACTS_DIR = ROOT / "docs" / "contracts"
GEN_DIR = ROOT / "docs" / "_generated" / "contracts"

# Explicit mapping for canonical hand-written pages when filename differs.
HANDWRITTEN_MAP = {
    "ERROR_CODES.json": "errors.md",
    "METRICS.json": "metrics.md",
    "TRACE_SPANS.json": "tracing.md",
    "ENDPOINTS.json": "endpoints.md",
    "CONFIG_KEYS.json": "config-keys.md",
    "CHART_VALUES.json": "chart-values.md",
}


def main() -> int:
    errors: list[str] = []
    for json_path in sorted(CONTRACTS_DIR.glob("*.json")):
        name = json_path.name
        generated = GEN_DIR / f"{json_path.stem}.md"
        hand_written = CONTRACTS_DIR / HANDWRITTEN_MAP.get(name, "")
        if generated.exists():
            continue
        if hand_written.name and hand_written.exists():
            continue
        errors.append(
            f"{json_path.relative_to(ROOT)} has no matching doc; expected {generated.relative_to(ROOT)} or mapped docs/contracts/*.md"
        )
    if errors:
        print("contract doc pair check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("contract doc pair check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
