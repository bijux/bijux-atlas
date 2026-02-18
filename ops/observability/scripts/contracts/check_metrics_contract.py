#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/observability/contract/metrics-contract.json"
METRICS_SRC = ROOT / "crates/bijux-atlas-server/src/telemetry/metrics_endpoint.rs"

contract = json.loads(CONTRACT.read_text())
required = contract.get("required_metrics", {})
if not isinstance(required, dict) or not required:
    print("metrics contract has no required_metrics", file=sys.stderr)
    sys.exit(1)

src = METRICS_SRC.read_text()
exported = set(re.findall(r'\b(?:bijux|atlas)_[a-z0-9_]+(?=\{)', src))

missing = [m for m in sorted(required.keys()) if m not in exported]
if missing:
    print("required metrics missing from exporter:", file=sys.stderr)
    for m in missing:
        print(f"- {m}", file=sys.stderr)
    sys.exit(1)

# Cardinality guardrail: disallow user-controlled labels.
for metric, labels in sorted(required.items()):
    if not isinstance(labels, list):
        print(f"metric {metric} labels must be a list", file=sys.stderr)
        sys.exit(1)
    forbidden = sorted(set(labels).intersection({"gene_id", "name", "prefix", "cursor", "region", "ip"}))
    if forbidden:
        print(f"metric {metric} has forbidden labels: {', '.join(forbidden)}", file=sys.stderr)
        sys.exit(1)

print("metrics contract passed")
