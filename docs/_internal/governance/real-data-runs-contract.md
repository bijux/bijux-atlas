# Real Data Runs Contract

The real data runs catalog is the source of truth for the ten governed reproducibility runs.

Required source file: `configs/tutorials/real-data-runs.json`
Required schema file: `configs/contracts/real-data-runs.schema.json`

Required fields per run:
- `id`: stable run identifier.
- `run_label`: human-readable durable label used in docs tables.
- `dataset`, `dataset_size_tier`, `ingest_mode`, `expected_outputs`.
- `input_provenance`: `url`, `retrieval_method`, `license_note`.
- `expected_query_set`: list of named query definitions.
- `expected_artifacts`: artifacts that must exist after execution.
- `expected_resource_profile`: runtime resource class.
- `expected_runtime_compatibility`: supported runtime version range.

Execution expectations for the first implementation slice:
- `tutorials real-data list`: emit stable run inventory.
- `tutorials real-data plan --run-id <id>`: emit deterministic execution plan.
- `tutorials real-data fetch --run-id <id>`: write dataset cache payload, checksum manifest, and provenance record.
- `tutorials real-data ingest --run-id <id>`: produce run-scoped ingest report under `artifacts/tutorials/runs/<run_id>/`.
- `tutorials real-data doctor`: verify cache integrity surfaces for every run.
- `tutorials real-data compare-regression --run-id <id>`: compare run summaries against golden summaries using policy thresholds.
- `tutorials real-data verify-idempotency --run-id <id>`: verify ingest and query outputs are hash-stable across reruns.
- `docs generate real-data-pages --allow-write`: generate the runs table and overview pages under `docs/`.

Regression and runtime policy files:
- `configs/tutorials/regression-threshold-policy.json`
- `configs/tutorials/data-source-volatility-policy.json`
- `configs/tutorials/cache-eviction-policy.json`
- `configs/tutorials/fetch-retry-policy.json`

Reader-facing generated outputs:
- `docs/_generated/real-data-runs-table.md`
- `docs/_internal/generated/real-data-runs-overview.md`
- `docs/_internal/generated/docs-artifact-link-inventory.md`
