# Script Entrypoints

This registry defines script stability levels.

- `public`: may be invoked by `make` recipes.
- `internal`: callable from other scripts only.
- `private`: local helper, not part of standard workflows.

## Public

- `scripts/areas/public/*`
- `scripts/areas/check/*`
- `scripts/areas/gen/*`
- `scripts/bin/isolate`
- `scripts/bin/require-isolate`
- `scripts/bin/bijux-atlas-dev`
- `scripts/bin/bijux-atlas-scripts`
- `scripts/areas/bootstrap/install_tools.sh`
- `scripts/areas/contracts/check_all.sh`
- `scripts/areas/contracts/check_chart_values_contract.py`
- `scripts/areas/contracts/generate_chart_values_schema.py`
- `scripts/areas/contracts/generate_contract_artifacts.py`
- `scripts/areas/configs/*`
- `scripts/areas/docs/*`
- `scripts/areas/fixtures/fetch-medium.sh`
- `scripts/areas/fixtures/fetch-real-datasets.sh`
- `scripts/areas/fixtures/run-medium-ingest.sh`
- `scripts/areas/fixtures/run-medium-serve.sh`
- `scripts/areas/gen/generate_scripts_readme.py`
- `scripts/areas/layout/check_no_root_dumping.sh`
- `scripts/areas/layout/*`
- `scripts/areas/public/observability/*`
- `scripts/areas/release/update-compat-matrix.sh`
- `scripts/areas/release/validate-compat-matrix.sh`
- `scripts/areas/public/perf/*` (compat wrappers only; canonical implementations live in `ops/load/scripts/`)
- `scripts/areas/ops/check_k8s_test_contract.py`
- `scripts/areas/ops/check_k8s_flakes.py`
- `scripts/areas/ops/check_k8s_checks_layout.py`
- `scripts/areas/ops/check_k8s_test_lib.py`
- `scripts/areas/ops/generate_k8s_test_surface.py`

## Internal

- `scripts/areas/internal/*`
- `scripts/lib/*`
- `scripts/areas/python/*`
- `ops/load/scripts/*`
- `ops/obs/scripts/*`
- `ops/stack/scripts/*`
- `scripts/areas/_internal/*`

## Private

- `scripts/areas/dev/*`
- `scripts/areas/demo/demo.sh`
