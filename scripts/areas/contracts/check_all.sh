#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

./scripts/areas/contracts/format_contracts.py
./scripts/areas/contracts/generate_contract_artifacts.py
./scripts/areas/contracts/generate_chart_values_schema.py
./scripts/areas/contracts/check_error_codes_contract.py
./scripts/areas/contracts/check_endpoints_contract.py
./scripts/areas/contracts/check_chart_values_contract.py
./scripts/areas/contracts/check_cli_ssot.py
./scripts/areas/contracts/check_config_keys_contract.py
./scripts/areas/contracts/check_contract_drift.py
./scripts/areas/contracts/check_breaking_contract_change.py

if ! git diff --quiet -- docs/_generated/contracts crates/bijux-atlas-api/src/generated crates/bijux-atlas-server/src/telemetry/generated ops/obs/contract/metrics-contract.json ops/k8s/charts/bijux-atlas/values.schema.json; then
  echo "generated contract artifacts are stale; run scripts/areas/contracts/generate_contract_artifacts.py and commit" >&2
  git --no-pager diff -- docs/_generated/contracts crates/bijux-atlas-api/src/generated crates/bijux-atlas-server/src/telemetry/generated ops/obs/contract/metrics-contract.json ops/k8s/charts/bijux-atlas/values.schema.json >&2 || true
  exit 1
fi

echo "ssot contract checks passed"
