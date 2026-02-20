#!/usr/bin/env sh
# owner: platform
# purpose: run policy lint suite and policy-adjacent script checks.
# stability: public
# called-by: make policy-lint, make _lint-configs
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
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
    "mode",
    "allow_override",
    "network_in_unit_tests",
    "modes",
    "query_budget",
    "response_budget",
    "cache_budget",
    "store_resilience",
    "rate_limit",
    "concurrency_bulkheads",
    "telemetry",
    "publish_gates",
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
for item in cfg_data["documented_defaults"]:
    if not isinstance(item, dict):
        raise SystemExit("documented_defaults entries must be objects")
    if not isinstance(item.get("field"), str) or not item["field"].strip():
        raise SystemExit("documented_defaults.field must be non-empty string")
    if not isinstance(item.get("reason"), str) or not item["reason"].strip():
        raise SystemExit("documented_defaults.reason must be non-empty string")

for section, keys in {
    "query_budget": {"cheap", "medium", "heavy", "max_limit", "max_prefix_length"},
    "response_budget": {"cheap_max_bytes", "medium_max_bytes", "heavy_max_bytes", "max_serialization_bytes"},
    "cache_budget": {"max_disk_bytes", "max_dataset_count", "pinned_datasets_max"},
    "store_resilience": {"retry_budget", "retry_attempts", "breaker_failure_threshold"},
    "rate_limit": {"per_ip_rps", "per_api_key_rps"},
    "concurrency_bulkheads": {"cheap", "medium", "heavy"},
    "telemetry": {"metrics_enabled", "tracing_enabled", "slow_query_log_enabled", "request_id_required", "required_metric_labels", "trace_sampling_per_10k"},
    "publish_gates": {"required_indexes", "min_gene_count", "max_missing_parents"},
}.items():
    block = cfg_data.get(section)
    if not isinstance(block, dict):
        raise SystemExit(f"{section} must be object")
    missing_block = sorted(keys - set(block.keys()))
    if missing_block:
        raise SystemExit(f"{section} missing keys: {missing_block}")

if cfg_data["mode"] not in {"strict", "compat", "dev"}:
    raise SystemExit("mode must be one of strict|compat|dev")
if not isinstance(cfg_data["modes"], dict):
    raise SystemExit("modes must be object")
for mode_name in ("strict", "compat", "dev"):
    mode = cfg_data["modes"].get(mode_name)
    if not isinstance(mode, dict):
        raise SystemExit(f"modes.{mode_name} must be object")
    for key in ("allow_override", "max_page_size", "max_region_span", "max_response_bytes"):
        if key not in mode:
            raise SystemExit(f"modes.{mode_name}.{key} is required")
    if int(mode["max_page_size"]) <= 0 or int(mode["max_region_span"]) <= 0 or int(mode["max_response_bytes"]) <= 0:
        raise SystemExit(f"modes.{mode_name} cap values must be > 0")
if cfg_data["modes"]["strict"]["allow_override"] is not False:
    raise SystemExit("modes.strict.allow_override must be false")
if cfg_data["modes"]["dev"]["allow_override"] is not False:
    raise SystemExit("modes.dev.allow_override must be false; overrides are compat-only")

print("policy config validated")
PY

./scripts/areas/public/require-crate-docs.sh
./scripts/areas/public/no-network-unit-tests.sh
./scripts/areas/public/check-cli-commands.sh
./scripts/areas/public/policy-schema-drift.py
./scripts/areas/public/check-allow-env-schema.py
./bin/bijux-atlas contracts generate --generators artifacts chart-schema
./bin/bijux-atlas contracts check --checks breakage drift endpoints error-codes sqlite-indexes chart-values
./scripts/areas/internal/effects-lint.sh
./scripts/areas/internal/naming-intent-lint.sh
