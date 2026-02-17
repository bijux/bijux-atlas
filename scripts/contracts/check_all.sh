#!/usr/bin/env sh
set -eu

./scripts/contracts/format_contracts.py
./scripts/contracts/generate_contract_artifacts.py
./scripts/contracts/check_error_codes_contract.py
./scripts/contracts/check_endpoints_contract.py
./scripts/contracts/check_chart_values_contract.py
./scripts/contracts/check_cli_ssot.py
./scripts/contracts/check_config_keys_contract.py
./scripts/contracts/check_contract_drift.py
./scripts/contracts/check_breaking_contract_change.py

if ! git diff --quiet -- docs/contracts/generated crates/bijux-atlas-api/src/generated crates/bijux-atlas-server/src/telemetry/generated observability/metrics_contract.json; then
  echo "generated contract artifacts are stale; run scripts/contracts/generate_contract_artifacts.py and commit" >&2
  git --no-pager diff -- docs/contracts/generated crates/bijux-atlas-api/src/generated crates/bijux-atlas-server/src/telemetry/generated observability/metrics_contract.json >&2 || true
  exit 1
fi

echo "ssot contract checks passed"
