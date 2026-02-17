#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "ops/observability/contract/metrics-contract.json"
DASH_CONTRACT = ROOT / "ops/observability/contract/dashboard-panels-contract.json"
DASH = ROOT / "ops/observability/grafana/atlas-observability-dashboard.json"

contract = json.loads(CONTRACT.read_text())
required = set(contract.get("required_metrics", {}).keys())
allow = required | {"bijux_cheap_queries_served_while_overloaded_total"}

dash = json.loads(DASH.read_text())
if not isinstance(dash.get("schemaVersion"), int) or dash.get("schemaVersion", 0) <= 0:
    print("dashboard missing positive schemaVersion", file=sys.stderr)
    sys.exit(1)
if not isinstance(dash.get("version"), int) or dash.get("version", 0) <= 0:
    print("dashboard missing positive version", file=sys.stderr)
    sys.exit(1)
text = json.dumps(dash)
metrics = set(re.findall(r'\b(?:bijux|atlas)_[a-z0-9_]+\b', text))
unknown = sorted(metrics - allow)
if unknown:
    print("dashboard references metrics not in metrics contract:", file=sys.stderr)
    for m in unknown:
        print(f"- {m}", file=sys.stderr)
    sys.exit(1)

# required SLO burn-rate panel presence
panel_titles = set()
for panel in dash.get("panels", []):
    title = panel.get("title")
    if isinstance(title, str):
        panel_titles.add(title)

required_panels = set(json.loads(DASH_CONTRACT.read_text()).get("required_panels", []))
missing_panels = sorted(required_panels - panel_titles)
if missing_panels:
    print("missing required dashboard panels:", file=sys.stderr)
    for panel in missing_panels:
        print(f"- {panel}", file=sys.stderr)
    sys.exit(1)

dash_contract = json.loads(DASH_CONTRACT.read_text())
contract_sha = dash_contract.get("contract_git_sha")
if not isinstance(contract_sha, str) or not contract_sha:
    print("dashboard panels contract missing contract_git_sha", file=sys.stderr)
    sys.exit(1)
tag_key = "contract_git_sha:"
tags = dash.get("tags", [])
if not any(isinstance(t, str) and t.startswith(tag_key) for t in tags):
    print("dashboard missing contract_git_sha tag", file=sys.stderr)
    sys.exit(1)

print("dashboard contract passed")
