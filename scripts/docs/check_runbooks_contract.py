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

ROOT = Path(__file__).resolve().parents[2]
RUNBOOK_DIR = ROOT / "docs" / "operations" / "runbooks"
REQUIRED_SECTIONS = [
    "Symptoms",
    "Metrics",
    "Commands",
    "Expected outputs",
    "Mitigations",
    "Rollback",
    "Postmortem checklist",
]

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

    if "operations/observability/dashboard.md" not in text and "../observability/dashboard.md" not in text:
        errors.append(f"{path}: missing dashboard link to operations/observability/dashboard.md")
    if not re.search(r"ops-drill-[a-z0-9-]+", text):
        errors.append(f"{path}: missing drill make target reference (ops-drill-*)")

if errors:
    print("runbook contract check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    sys.exit(1)

print("runbook contract check passed")
