#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/observability/contract/metrics-contract.json"
DASH_CONTRACT = ROOT / "ops/observability/contract/dashboard-panels-contract.json"
DASH = ROOT / "ops/observability/grafana/atlas-observability-dashboard.json"
GOLDEN = ROOT / "ops/observability/grafana/atlas-observability-dashboard.golden.json"

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

# Validate panel query expressions are present and contract-safe.
errors: list[str] = []
expr_metric_re = re.compile(r'\b(?:bijux|atlas)_[a-z0-9_]+\b')
forbidden_high_cardinality = {"gene_id", "tx_id", "request_id", "trace_id", "dataset_id", "name", "cursor"}
for panel in dash.get("panels", []):
    title = panel.get("title") if isinstance(panel.get("title"), str) else "<untitled>"
    panel_type = panel.get("type")
    if panel_type != "row":
        desc = panel.get("description")
        if not isinstance(desc, str) or not desc.strip().endswith("?"):
            errors.append(f"panel '{title}' must include a diagnostic question description ending with '?'")
    for target in panel.get("targets", []):
        expr = target.get("expr")
        if expr is None:
            continue
        if not isinstance(expr, str) or not expr.strip():
            errors.append(f"panel '{title}' has empty query expr")
            continue
        expr_lower = expr.lower()
        if "todo" in expr_lower or "placeholder" in expr_lower:
            errors.append(f"panel '{title}' contains placeholder/todo query")
            continue
        expr_metrics = set(expr_metric_re.findall(expr))
        unknown_expr = sorted(expr_metrics - allow)
        for metric_name in unknown_expr:
            errors.append(
                f"panel '{title}' query references metric not in metrics contract: {metric_name}"
            )
        if re.search(r"[{,]\s*(?:route|dataset|request_id|trace_id)\s*=~", expr):
            errors.append(f"panel '{title}' uses regex on high-cardinality labels")
        if re.search(r".*=~\\\"\\.\\*\\\"", expr):
            errors.append(f"panel '{title}' uses wildcard regex selector")
        for label in forbidden_high_cardinality:
            if re.search(rf"[{{,]\s*{re.escape(label)}\s*=", expr):
                errors.append(f"panel '{title}' filters on forbidden high-cardinality label `{label}`")

if errors:
    for err in errors:
        print(err, file=sys.stderr)
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
required_rows = set(json.loads(DASH_CONTRACT.read_text()).get("required_rows", []))
present_rows = {
    panel.get("title")
    for panel in dash.get("panels", [])
    if isinstance(panel, dict) and panel.get("type") == "row" and isinstance(panel.get("title"), str)
}
missing_rows = sorted(required_rows - present_rows)
if missing_rows:
    print("missing required dashboard rows:", file=sys.stderr)
    for row in missing_rows:
        print(f"- {row}", file=sys.stderr)
    sys.exit(1)

dash_contract = json.loads(DASH_CONTRACT.read_text())
contract_sha = dash_contract.get("contract_git_sha")
if not isinstance(contract_sha, str) or not contract_sha:
    print("dashboard panels contract missing contract_git_sha", file=sys.stderr)
    sys.exit(1)
panel_specs = dash_contract.get("panel_specs", {})
if not isinstance(panel_specs, dict):
    print("dashboard panels contract missing panel_specs", file=sys.stderr)
    sys.exit(1)
missing_panel_specs = sorted(required_panels - set(panel_specs.keys()))
if missing_panel_specs:
    print("dashboard contract missing panel_specs entries:", file=sys.stderr)
    for panel in missing_panel_specs:
        print(f"- {panel}", file=sys.stderr)
    sys.exit(1)
for panel, spec in sorted(panel_specs.items()):
    for field in ("diagnostic_question", "metrics", "failure_signatures"):
        if field not in spec:
            print(f"panel spec {panel} missing field: {field}", file=sys.stderr)
            sys.exit(1)
    if not isinstance(spec["diagnostic_question"], str) or not spec["diagnostic_question"].strip().endswith("?"):
        print(f"panel spec {panel} must provide diagnostic_question ending with '?'", file=sys.stderr)
        sys.exit(1)
    if not isinstance(spec["metrics"], list) or not isinstance(spec["failure_signatures"], list):
        print(f"panel spec {panel} metrics/failure_signatures must be lists", file=sys.stderr)
        sys.exit(1)
    if not spec["metrics"] or not spec["failure_signatures"]:
        print(f"panel spec {panel} must include at least one metric and one failure signature", file=sys.stderr)
        sys.exit(1)
    for metric_name in spec["metrics"]:
        if metric_name not in required:
            print(f"panel spec {panel} references unknown metric: {metric_name}", file=sys.stderr)
            sys.exit(1)
tag_key = "contract_git_sha:"
tags = dash.get("tags", [])
if not any(isinstance(t, str) and t.startswith(tag_key) for t in tags):
    print("dashboard missing contract_git_sha tag", file=sys.stderr)
    sys.exit(1)

# no orphan dashboards: every panel must be referenced in runbook-dashboard-alert map doc
runbook_map = (ROOT / "docs/operations/observability/runbook-dashboard-alert-map.md").read_text()
for panel in sorted(required_panels):
    if panel not in runbook_map:
        print(f"runbook map missing dashboard panel reference: {panel}", file=sys.stderr)
        sys.exit(1)


def normalized_dashboard(payload: dict) -> dict:
    out = json.loads(json.dumps(payload))
    out["version"] = 0
    out["schemaVersion"] = 0
    out["tags"] = sorted(
        [t for t in out.get("tags", []) if isinstance(t, str) and not t.startswith("contract_git_sha:")]
    )
    return out


current_norm = normalized_dashboard(dash)
if not GOLDEN.exists():
    GOLDEN.write_text(json.dumps(current_norm, indent=2) + "\n")
expected_norm = json.loads(GOLDEN.read_text())
if current_norm != expected_norm:
    print("dashboard golden snapshot drift detected", file=sys.stderr)
    print(f"expected: {GOLDEN}", file=sys.stderr)
    print(f"actual_sha256={hashlib.sha256(json.dumps(current_norm, sort_keys=True).encode()).hexdigest()}", file=sys.stderr)
    print(f"golden_sha256={hashlib.sha256(json.dumps(expected_norm, sort_keys=True).encode()).hexdigest()}", file=sys.stderr)
    sys.exit(1)

print("dashboard contract passed")
