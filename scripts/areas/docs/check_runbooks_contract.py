#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUNBOOK_DIR = ROOT / "docs" / "operations" / "runbooks"
REQUIRED_SECTIONS = [
    "Symptoms",
    "Metrics",
    "Commands",
    "Expected outputs",
    "Mitigations",
    "Alerts",
    "Rollback",
    "Postmortem checklist",
]
ALERT_NAMES = set(
    json.loads((ROOT / "ops" / "obs" / "contract" / "alerts-contract.json").read_text()).get("required_alerts", [])
)

metrics = {
    m["name"] for m in json.loads((ROOT / "docs" / "contracts" / "METRICS.json").read_text())["metrics"]
}
endpoint_registry = {e["path"] for e in json.loads((ROOT / "docs" / "contracts" / "ENDPOINTS.json").read_text())["endpoints"]}
endpoint_registry.update({"/metrics", "/healthz", "/readyz", "/debug/datasets", "/debug/registry-health"})

make_db = subprocess.run(["make", "-qp"], cwd=ROOT, text=True, capture_output=True, check=False).stdout
make_targets = set(re.findall(r"^([A-Za-z0-9_.%/+\-]+):", make_db, flags=re.MULTILINE))

errors: list[str] = []
for path in sorted(RUNBOOK_DIR.glob("*.md")):
    if path.name == "INDEX.md":
        continue
    text = path.read_text(encoding="utf-8")
    for sec in REQUIRED_SECTIONS:
        if not re.search(rf"^##\s+{re.escape(sec)}\s*$", text, flags=re.MULTILINE):
            errors.append(f"{path}: missing section '## {sec}'")

    for metric in re.findall(r"`(bijux_[a-z0-9_]+)`", text):
        if metric not in metrics:
            errors.append(f"{path}: unknown metric `{metric}`")

    for ep in re.findall(r"(/(?:v1|metrics|healthz|readyz|debug)[a-zA-Z0-9_\-/{}:?=&.]*)", text):
        ep0 = ep.split("?")[0]
        if ep0 not in endpoint_registry:
            errors.append(f"{path}: unknown endpoint `{ep0}`")

    for cmd in re.findall(r"^\$\s+(.+)$", text, flags=re.MULTILINE):
        if cmd.startswith("make "):
            target = cmd.split()[1]
            if target not in make_targets:
                errors.append(f"{path}: unknown make target `{target}`")

    obs_dir = "observ" + "ability"
    dashboard_pattern = rf"(docs/operations/{obs_dir}/dashboard\.md|\.\./{obs_dir}/dashboard\.md)"
    if not re.search(dashboard_pattern, text):
        errors.append(f"{path}: missing dashboard link to observability dashboard")
    if not re.search(r"ops-drill-[a-z0-9-]+", text):
        errors.append(f"{path}: missing drill make target reference (ops-drill-*)")
    alert_refs = re.findall(r"`([A-Za-z][A-Za-z0-9]+)`", text)
    listed_alerts = sorted(set(a for a in alert_refs if a in ALERT_NAMES))
    if not listed_alerts:
        errors.append(f"{path}: Alerts section must list at least one known alert id")

# bidirectional map contract: every alert appears in runbook map and every runbook appears with at least one alert.
map_doc = (ROOT / "docs/operations/observability/runbook-dashboard-alert-map.md").read_text(encoding="utf-8")
for alert in sorted(ALERT_NAMES):
    if alert not in map_doc:
        errors.append(f"runbook-dashboard-alert-map: missing alert `{alert}`")
for path in sorted(RUNBOOK_DIR.glob("*.md")):
    if path.name == "INDEX.md":
        continue
    if path.name not in map_doc:
        errors.append(f"runbook-dashboard-alert-map: missing runbook row for `{path.name}`")

if errors:
    print("runbook contract check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    sys.exit(1)

print("runbook contract check passed")
