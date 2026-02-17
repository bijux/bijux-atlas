# Script Entrypoints

This registry defines script stability levels.

- `public`: may be invoked by `make` recipes.
- `internal`: callable from other scripts only.
- `private`: local helper, not part of standard workflows.

## Public

- `scripts/public/*`
- `scripts/bin/isolate`
- `scripts/bin/require-isolate`
- `scripts/bootstrap/install_tools.sh`
- `scripts/check-cli-commands.sh`
- `scripts/check-markdown-links.sh`
- `scripts/contracts/check_all.sh`
- `scripts/contracts/check_chart_values_contract.py`
- `scripts/contracts/generate_chart_values_schema.py`
- `scripts/contracts/generate_contract_artifacts.py`
- `scripts/docs/*`
- `scripts/fixtures/fetch-medium.sh`
- `scripts/fixtures/fetch-real-datasets.sh`
- `scripts/fixtures/run-medium-ingest.sh`
- `scripts/fixtures/run-medium-serve.sh`
- `scripts/generate_scripts_readme.py`
- `scripts/layout/*`
- `scripts/no-network-unit-tests.sh`
- `scripts/observability/*`
- `scripts/openapi-diff-check.sh`
- `scripts/policy-lint.sh`
- `scripts/policy-schema-drift.py`
- `scripts/query-plan-gate.sh`
- `scripts/release/update-compat-matrix.sh`
- `scripts/release/validate-compat-matrix.sh`
- `scripts/require-crate-docs.sh`
- `scripts/perf/*` (compat wrappers only; canonical implementations live in `ops/load/scripts/`)
- `scripts/ops/check_k8s_test_contract.py`
- `scripts/ops/check_k8s_flakes.py`

## Internal

- `scripts/internal/*`
- `ops/load/scripts/*`
- `ops/observability/scripts/*`
- `ops/stack/scripts/*`
- `scripts/_internal/*`

## Private

- `scripts/dev/*`
- `scripts/demo/demo.sh`
