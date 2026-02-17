#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "observability/metrics_contract.json"
DASH = ROOT / "observability/grafana/atlas-observability-dashboard.json"

contract = json.loads(CONTRACT.read_text())
required = set(contract.get("required_metrics", {}).keys())
allow = required | {"bijux_cheap_queries_served_while_overloaded_total"}

dash = json.loads(DASH.read_text())
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

if "SLO Burn Rate (5xx, 5m/1h)" not in panel_titles:
    print("missing required SLO burn-rate panel", file=sys.stderr)
    sys.exit(1)

print("dashboard contract passed")
