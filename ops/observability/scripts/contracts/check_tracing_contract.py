#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/observability/contract/metrics-contract.json"

contract = json.loads(CONTRACT.read_text())
required = contract.get("required_spans", [])
if not required:
    print("no required_spans in contract", file=sys.stderr)
    sys.exit(1)

corpus = "\n".join(
    p.read_text()
    for p in (ROOT / "crates/bijux-atlas-server/src").rglob("*.rs")
)

missing = [s for s in required if s not in corpus]
if missing:
    print("required tracing spans/messages not found in source:", file=sys.stderr)
    for s in missing:
        print(f"- {s}", file=sys.stderr)
    sys.exit(1)

print("tracing contract passed")
