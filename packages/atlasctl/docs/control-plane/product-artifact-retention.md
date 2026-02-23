# Product Artifact Retention Policy

This policy defines what product artifacts are committed vs ephemeral.

## Committed

- Contracts and schemas (for example `configs/product/artifact-manifest.schema.json`)
- Small sample/golden manifests used for tests
- Generated docs only when explicitly updated via reviewed commands

## Ephemeral (Do Not Commit)

- Product build evidence under `artifacts/evidence/product/<lane>/<run_id>/...`
- Product reports under `artifacts/reports/atlasctl/...`
- Temporary scratch outputs under `packages/artifacts/tmp` (if used)

## Determinism Requirements

- `packages/artifacts/tmp` usage must be per-run (`<run_id>` namespaced) or explicitly cleaned
- Product artifact manifests must be reproducible for the same pins + inputs
- Artifact inventory rows and JSON output must be stable/sorted

## Cleanup

- Prefer writing to `artifacts/evidence/product/<run_id>` and deleting old runs by retention job or explicit cleanup command
- Do not reuse shared tmp paths across runs without a cleanup step
