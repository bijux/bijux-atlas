#!/usr/bin/env sh
set -eu

python3 - <<'PY'
import json
from pathlib import Path

cfg = Path("configs/policy/policy.json")
schema = Path("configs/policy/policy.schema.json")

if not cfg.exists() or not schema.exists():
    raise SystemExit("missing policy schema/config")

cfg_data = json.loads(cfg.read_text())
required = {
    "schema_version",
    "allow_override",
    "network_in_unit_tests",
    "query_budget",
    "cache_budget",
    "rate_limit",
    "concurrency_bulkheads",
    "telemetry",
    "documented_defaults",
}
missing = sorted(required - set(cfg_data.keys()))
if missing:
    raise SystemExit(f"missing required config keys: {missing}")

if cfg_data["allow_override"] is not False:
    raise SystemExit("allow_override must be false")
if cfg_data["network_in_unit_tests"] is not False:
    raise SystemExit("network_in_unit_tests must be false")
if not isinstance(cfg_data["documented_defaults"], list):
    raise SystemExit("documented_defaults must be list")

for section, keys in {
    "query_budget": {"max_limit", "max_region_span", "max_prefix_length"},
    "cache_budget": {"max_disk_bytes", "max_dataset_count", "pinned_datasets_max"},
    "rate_limit": {"per_ip_rps", "per_api_key_rps"},
    "concurrency_bulkheads": {"cheap", "medium", "heavy"},
    "telemetry": {"metrics_enabled", "tracing_enabled", "slow_query_log_enabled", "request_id_required"},
}.items():
    block = cfg_data.get(section)
    if not isinstance(block, dict):
        raise SystemExit(f"{section} must be object")
    missing_block = sorted(keys - set(block.keys()))
    if missing_block:
        raise SystemExit(f"{section} missing keys: {missing_block}")

print("policy config validated")
PY

./scripts/require-crate-docs.sh
./scripts/no-network-unit-tests.sh
./scripts/check-cli-commands.sh
