#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/obs/contract/metrics-contract.json"
METRICS_SRC = ROOT / "crates/bijux-atlas-server/src/telemetry/metrics_endpoint.rs"

contract = json.loads(CONTRACT.read_text())
required = contract.get("required_metrics", {})
if not isinstance(required, dict) or not required:
    print("metrics contract has no required_metrics", file=sys.stderr)
    sys.exit(1)
specs = contract.get("required_metric_specs", {})
if not isinstance(specs, dict) or set(specs) != set(required):
    print("metrics contract required_metric_specs must exist and cover exactly required_metrics", file=sys.stderr)
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
    spec = specs.get(metric, {})
    required_fields = {
        "type",
        "unit",
        "cardinality_budget",
        "required_labels",
        "forbidden_labels",
        "example_series",
        "semantic",
        "owner",
        "slo_relevance",
        "criticality",
    }
    missing_fields = sorted(required_fields - set(spec.keys()))
    if missing_fields:
        print(f"metric {metric} missing spec fields: {', '.join(missing_fields)}", file=sys.stderr)
        sys.exit(1)
    if spec["type"] not in {"counter", "gauge", "histogram"}:
        print(f"metric {metric} has invalid type: {spec['type']}", file=sys.stderr)
        sys.exit(1)
    if not isinstance(spec["unit"], str) or not spec["unit"]:
        print(f"metric {metric} has invalid unit", file=sys.stderr)
        sys.exit(1)
    if spec["required_labels"] != labels:
        print(f"metric {metric} required_labels must match required_metrics labels", file=sys.stderr)
        sys.exit(1)
    budget = spec["cardinality_budget"]
    if (
        not isinstance(budget, dict)
        or not isinstance(budget.get("max_series"), int)
        or not isinstance(budget.get("max_new_series_per_hour"), int)
    ):
        print(f"metric {metric} invalid cardinality_budget", file=sys.stderr)
        sys.exit(1)
    forbidden = sorted(set(labels).intersection(set(spec["forbidden_labels"])))
    if forbidden:
        print(f"metric {metric} has forbidden labels: {', '.join(forbidden)}", file=sys.stderr)
        sys.exit(1)
    sem = spec["semantic"]
    if not isinstance(sem, dict) or not sem.get("what_it_measures") or not sem.get("on_break_action"):
        print(f"metric {metric} semantic metadata incomplete", file=sys.stderr)
        sys.exit(1)
    owner = spec["owner"]
    if not isinstance(owner, dict) or not owner.get("crate") or not owner.get("module"):
        print(f"metric {metric} owner metadata incomplete", file=sys.stderr)
        sys.exit(1)
    slo = spec["slo_relevance"]
    if not isinstance(slo, dict) or not isinstance(slo.get("relevant"), bool) or not isinstance(slo.get("slos"), list):
        print(f"metric {metric} slo_relevance metadata invalid", file=sys.stderr)
        sys.exit(1)
    if spec["criticality"] not in {"tier-0", "tier-1", "tier-2"}:
        print(f"metric {metric} criticality must be tier-0|tier-1|tier-2", file=sys.stderr)
        sys.exit(1)
    if not isinstance(spec["example_series"], str) or metric not in spec["example_series"]:
        print(f"metric {metric} example_series invalid", file=sys.stderr)
        sys.exit(1)

print("metrics contract passed")
