#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT = ROOT / "ops/observability/metrics_contract.json"
ALERTS = ROOT / "ops/observability/alerts/atlas-alert-rules.yaml"

contract = json.loads(CONTRACT.read_text())
required = set(contract.get("required_metrics", {}).keys())
allow = required | {
    "bijux_store_open_failure_total",
    "bijux_store_download_failure_total",
}

text = ALERTS.read_text()
alerts = re.findall(r"^\s*-\s*alert:\s*(\S+)\s*$", text, flags=re.MULTILINE)
if not alerts:
    print("no alerts found", file=sys.stderr)
    sys.exit(1)

required_alerts = {
    "BijuxAtlasHigh5xxRate",
    "BijuxAtlasP95LatencyRegression",
    "BijuxAtlasStoreDownloadFailures",
    "BijuxAtlasCacheThrash",
}
missing_alerts = sorted(required_alerts - set(alerts))
if missing_alerts:
    print("missing required alerts:", file=sys.stderr)
    for a in missing_alerts:
        print(f"- {a}", file=sys.stderr)
    sys.exit(1)

metrics = set(re.findall(r"\b(?:bijux|atlas)_[a-zA-Z0-9_]+\b", text))
unknown = sorted(metrics - allow)
if unknown:
    print("alert rules reference unknown metrics:", file=sys.stderr)
    for m in unknown:
        print(f"- {m}", file=sys.stderr)
    sys.exit(1)

# Unit-like check: ensure high-5xx alert expression has a threshold comparator.
high_5xx_block = re.search(
    r"alert:\s*BijuxAtlasHigh5xxRate[\s\S]*?expr:\s*\|([\s\S]*?)\n\s*for:",
    text,
)
if not high_5xx_block or ">" not in high_5xx_block.group(1):
    print("high 5xx alert expression missing threshold comparator", file=sys.stderr)
    sys.exit(1)

# Unit-like simulation: high_5xx should fire on a bad ratio and not fire on a healthy ratio.
th = re.search(r">\s*([0-9.]+)", high_5xx_block.group(1))
if not th:
    print("high 5xx alert threshold not parseable", file=sys.stderr)
    sys.exit(1)
threshold = float(th.group(1))
healthy_ratio = 0.001
bad_ratio = 0.02
if healthy_ratio > threshold:
    print("high 5xx alert would fire in healthy scenario", file=sys.stderr)
    sys.exit(1)
if bad_ratio <= threshold:
    print("high 5xx alert would not fire in bad scenario", file=sys.stderr)
    sys.exit(1)

print("alerts contract passed")