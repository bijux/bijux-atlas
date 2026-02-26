# Load Result Contract

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines mandatory shape and metadata for load result artifacts.

## Why

Ensures downstream scoring and regression checks consume deterministic outputs.

## Contracts

- Schema: `ops/load/contracts/result-schema.json`
- Validator: `crates/bijux-dev-atlas/src/commands/ops/load/reports/validate_results.py` (invoke via `bijux dev atlas`)
- Required sidecar metadata fields:
  - `git_sha`
  - `image_digest`
  - `dataset_hash`
  - `dataset_release`
  - `base_url`

## How to verify

```bash
$ make ops-load-smoke
$ make ops-load-smoke
```

Expected output: load result contract validation passes.

## See also

- [Load Suites](suites.md)
- [Load Reproducibility](reproducibility.md)
- `ops-load-full`

- Reference scenario: `mixed.json`
