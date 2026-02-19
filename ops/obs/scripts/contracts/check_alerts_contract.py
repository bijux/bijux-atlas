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
ALERT_CONTRACT = ROOT / "ops/obs/contract/alerts-contract.json"
DRILLS_MANIFEST = ROOT / "ops/obs/drills/drills.json"
ALERT_RULE_FILES = [
    ROOT / "ops/obs/alerts/atlas-alert-rules.yaml",
    ROOT / "ops/obs/alerts/slo-burn-rules.yaml",
]

contract = json.loads(CONTRACT.read_text())
required = set(contract.get("required_metrics", {}).keys())
allow = required | {
    "bijux_store_open_failure_total",
    "bijux_store_download_failure_total",
}

texts = [p.read_text() for p in ALERT_RULE_FILES]
text = "\n".join(texts)
alerts = re.findall(r"^\s*-\s*alert:\s*(\S+)\s*$", text, flags=re.MULTILINE)
if not alerts:
    print("no alerts found", file=sys.stderr)
    sys.exit(1)

required_alerts = set(json.loads(ALERT_CONTRACT.read_text()).get("required_alerts", []))
if not required_alerts:
    print("alerts contract missing required_alerts", file=sys.stderr)
    sys.exit(1)
missing_alerts = sorted(required_alerts - set(alerts))
if missing_alerts:
    print("missing required alerts:", file=sys.stderr)
    for a in missing_alerts:
        print(f"- {a}", file=sys.stderr)
    sys.exit(1)

alerts_contract = json.loads(ALERT_CONTRACT.read_text())
contract_sha = alerts_contract.get("contract_git_sha")
if not isinstance(contract_sha, str) or not contract_sha:
    print("alerts contract missing contract_git_sha", file=sys.stderr)
    sys.exit(1)
alert_specs = alerts_contract.get("alert_specs", {})
if not isinstance(alert_specs, dict):
    print("alerts contract missing alert_specs map", file=sys.stderr)
    sys.exit(1)
drills = json.loads(DRILLS_MANIFEST.read_text(encoding="utf-8")).get("drills", [])
drill_ids = {
    d.get("name")
    for d in drills
    if isinstance(d, dict) and isinstance(d.get("name"), str)
}

missing_specs = sorted(required_alerts - set(alert_specs.keys()))
if missing_specs:
    print("alerts contract missing alert_specs entries:", file=sys.stderr)
    for name in missing_specs:
        print(f"- {name}", file=sys.stderr)
    sys.exit(1)
for name, spec in sorted(alert_specs.items()):
    for field in ("slo_or_invariant", "runbook_id", "drill_id", "severity_tier", "owner"):
        if not isinstance(spec.get(field), str) or not spec.get(field):
            print(f"alert spec {name} missing field: {field}", file=sys.stderr)
            sys.exit(1)
    if spec["severity_tier"] not in {"info", "warn", "page"}:
        print(f"alert spec {name} has invalid severity_tier: {spec['severity_tier']}", file=sys.stderr)
        sys.exit(1)
    if spec["drill_id"] not in drill_ids:
        print(f"alert spec {name} references unknown drill_id: {spec['drill_id']}", file=sys.stderr)
        sys.exit(1)

metrics = set(re.findall(r"\b(?:bijux|atlas)_[a-zA-Z0-9_]+\b", text))
unknown = sorted(metrics - allow)
if unknown:
    print("alert rules reference unknown metrics:", file=sys.stderr)
    for m in unknown:
        print(f"- {m}", file=sys.stderr)
    sys.exit(1)

if "contract_version:" not in text:
    print("alert rules missing contract version metadata", file=sys.stderr)
    sys.exit(1)
if "contract_git_sha:" not in text:
    print("alert rules missing contract_git_sha metadata", file=sys.stderr)
    sys.exit(1)
if "contact:" not in text:
    print("alert rules missing contact annotation", file=sys.stderr)
    sys.exit(1)

for alert in required_alerts:
    block = re.search(
        rf"alert:\s*{re.escape(alert)}[\s\S]*?(?=\n\s*-\s*alert:|\Z)",
        text,
    )
    if not block:
        print(f"missing alert block: {alert}", file=sys.stderr)
        sys.exit(1)
    value = block.group(0)
    if "alert_contract_version:" not in value:
        print(f"missing alert_contract_version label for {alert}", file=sys.stderr)
        sys.exit(1)
    if "runbook:" not in value:
        print(f"missing runbook annotation for {alert}", file=sys.stderr)
        sys.exit(1)
    spec = alert_specs.get(alert, {})
    sev = re.search(r"severity:\s*([a-zA-Z]+)", value)
    if not sev:
        print(f"missing severity label for {alert}", file=sys.stderr)
        sys.exit(1)
    if sev.group(1).lower() != spec.get("severity_tier", "").lower():
        print(f"severity mismatch for {alert}: yaml={sev.group(1)} contract={spec.get('severity_tier')}", file=sys.stderr)
        sys.exit(1)
    runbook = re.search(r"runbook:\s*\"([^\"]+)\"", value)
    if not runbook:
        print(f"missing runbook path for {alert}", file=sys.stderr)
        sys.exit(1)
    expected_runbook = f"docs/operations/runbooks/{spec.get('runbook_id')}.md"
    if runbook.group(1) != expected_runbook:
        print(
            f"runbook mismatch for {alert}: yaml={runbook.group(1)} contract={expected_runbook}",
            file=sys.stderr,
        )
        sys.exit(1)

# no orphan alerts: every rule must be declared in contract
orphan_alerts = sorted(set(alerts) - required_alerts)
if orphan_alerts:
    print("orphan alerts found (not listed in alerts contract):", file=sys.stderr)
    for alert in orphan_alerts:
        print(f"- {alert}", file=sys.stderr)
    sys.exit(1)

# runbook mapping contract: every alert appears in runbook map table
runbook_map = (ROOT / "docs/operations/observability/runbook-dashboard-alert-map.md").read_text()
for alert in sorted(required_alerts):
    if alert not in runbook_map:
        print(f"runbook map missing alert reference: {alert}", file=sys.stderr)
        sys.exit(1)
for name, spec in sorted(alert_specs.items()):
    if spec["drill_id"] not in runbook_map:
        print(f"runbook map missing drill reference for {name}: {spec['drill_id']}", file=sys.stderr)
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
