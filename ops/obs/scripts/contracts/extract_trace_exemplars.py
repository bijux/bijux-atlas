#!/usr/bin/env python3
# Purpose: extract redacted exemplar trace ids grouped by scenario from trace artifacts.
# Inputs: artifacts/ops/obs/traces.exemplars.log
# Outputs: artifacts/ops/obs/trace-exemplars.by-scenario.json
from __future__ import annotations

import json
import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
IN_FILE = ROOT / "artifacts" / "ops" / "observability" / "traces.exemplars.log"
OUT_FILE = ROOT / "artifacts" / "ops" / "observability" / "trace-exemplars.by-scenario.json"

TRACE_ID_RE = re.compile(r"(?:trace[_-]?id)\s*[:=]\s*([a-fA-F0-9-]{8,64})")


def redact(value: str) -> str:
    if len(value) <= 8:
        return value
    return value[:4] + "..." + value[-4:]


def main() -> int:
    scenario = os.getenv("ATLAS_TRACE_SCENARIO", "default")
    OUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    if not IN_FILE.exists():
        payload = {"schema_version": 1, "scenarios": {scenario: []}}
        OUT_FILE.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {OUT_FILE}")
        return 0

    text = IN_FILE.read_text(encoding="utf-8", errors="replace")
    ids = sorted({redact(m.group(1)) for m in TRACE_ID_RE.finditer(text)})
    payload = {"schema_version": 1, "scenarios": {scenario: ids}}
    OUT_FILE.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT_FILE}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
