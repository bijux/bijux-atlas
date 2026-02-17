#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
contracts = ROOT / "docs" / "contracts"

errors = json.loads((contracts / "ERROR_CODES.json").read_text())
metrics = json.loads((contracts / "METRICS.json").read_text())
trace_spans = json.loads((contracts / "TRACE_SPANS.json").read_text())
endpoints = json.loads((contracts / "ENDPOINTS.json").read_text())
chart = json.loads((contracts / "CHART_VALUES.json").read_text())
config_keys = json.loads((contracts / "CONFIG_KEYS.json").read_text())
policy_schema = json.loads((contracts / "POLICY_SCHEMA.json").read_text())

# check sorted canonical
if errors["codes"] != sorted(errors["codes"]):
    print("ERROR_CODES.json codes not sorted", file=sys.stderr)
    sys.exit(1)
if chart["top_level_keys"] != sorted(chart["top_level_keys"]):
    print("CHART_VALUES.json top_level_keys not sorted", file=sys.stderr)
    sys.exit(1)
if config_keys["env_keys"] != sorted(config_keys["env_keys"]):
    print("CONFIG_KEYS.json env_keys not sorted", file=sys.stderr)
    sys.exit(1)
span_names = [s["name"] for s in trace_spans["spans"]]
if span_names != sorted(span_names):
    print("TRACE_SPANS.json spans not sorted", file=sys.stderr)
    sys.exit(1)
# error codes must match generated rust constants and openapi enum
rust_generated = (ROOT / "crates" / "bijux-atlas-api" / "src" / "generated" / "error_codes.rs").read_text()
for code in errors["codes"]:
    if f'"{code}"' not in rust_generated:
        print(f"missing generated rust error code: {code}", file=sys.stderr)
        sys.exit(1)

openapi_snapshot = json.loads((ROOT / "configs" / "openapi" / "v1" / "openapi.snapshot.json").read_text())
openapi_codes = (
    openapi_snapshot.get("components", {})
    .get("schemas", {})
    .get("ApiErrorCode", {})
    .get("enum", [])
)
if sorted(openapi_codes) != sorted(errors["codes"]):
    print("OpenAPI ApiErrorCode enum drift from ERROR_CODES.json", file=sys.stderr)
    sys.exit(1)

# metrics contract aligns with observability metrics contract
obs_metrics = json.loads((ROOT / "ops" / "observability" / "contract" / "metrics-contract.json").read_text())
obs_set = set(obs_metrics["required_metrics"].keys())
contract_set = {m["name"] for m in metrics["metrics"]}
if contract_set != obs_set:
    print("METRICS.json drift from ops/observability/contract/metrics-contract.json", file=sys.stderr)
    print("missing in METRICS:", sorted(obs_set - contract_set), file=sys.stderr)
    print("extra in METRICS:", sorted(contract_set - obs_set), file=sys.stderr)
    sys.exit(1)
obs_spans = obs_metrics.get("required_spans", [])
if sorted(obs_spans) != sorted(span_names):
    print("TRACE_SPANS.json drift from ops/observability/contract/metrics-contract.json", file=sys.stderr)
    sys.exit(1)

# endpoint registry matches server routes and openapi paths
server_src = (ROOT / "crates" / "bijux-atlas-server" / "src" / "runtime" / "server_runtime_app.rs").read_text()
route_paths = re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE)
route_set = set()
for p in route_paths:
    p = re.sub(r":([a-zA-Z_][a-zA-Z0-9_]*)", r"{\1}", p)
    if p != "/":
        route_set.add(p)
contract_paths = {e["path"] for e in endpoints["endpoints"]}
if route_set != contract_paths:
    print("ENDPOINTS.json drift from server routes", file=sys.stderr)
    print("missing in contract:", sorted(route_set - contract_paths), file=sys.stderr)
    print("extra in contract:", sorted(contract_paths - route_set), file=sys.stderr)
    sys.exit(1)
openapi_paths = set(openapi_snapshot.get("paths", {}).keys())
if contract_paths != openapi_paths:
    print("ENDPOINTS.json drift from OpenAPI paths", file=sys.stderr)
    print("missing in contract:", sorted(openapi_paths - contract_paths), file=sys.stderr)
    print("extra in contract:", sorted(contract_paths - openapi_paths), file=sys.stderr)
    sys.exit(1)

# chart values keys check
value_keys = sorted(
    {
        m.group(1)
        for m in re.finditer(
            r"^([A-Za-z][A-Za-z0-9_]*)\s*:",
            (ROOT / "ops" / "k8s" / "charts" / "bijux-atlas" / "values.yaml").read_text(),
            flags=re.MULTILINE,
        )
    }
)
if value_keys != sorted(chart["top_level_keys"]):
    print("CHART_VALUES.json drift from ops/k8s/charts/bijux-atlas/values.yaml", file=sys.stderr)
    print("missing in contract:", sorted(set(value_keys) - set(chart["top_level_keys"])), file=sys.stderr)
    print("extra in contract:", sorted(set(chart["top_level_keys"]) - set(value_keys)), file=sys.stderr)
    sys.exit(1)

workspace_policy_schema = json.loads(
    (ROOT / "configs" / "policy" / "policy.schema.json").read_text()
)
if policy_schema != workspace_policy_schema:
    print("POLICY_SCHEMA.json drift from configs/policy/policy.schema.json", file=sys.stderr)
    sys.exit(1)

print("contracts drift check passed")