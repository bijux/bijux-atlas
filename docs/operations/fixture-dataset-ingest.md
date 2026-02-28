# Fixture Dataset Ingest

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: provide the strict ingest recipe for fixture-backed datasets.

## Why you are reading this

Use this page when you need to ingest fixture datasets with deterministic inputs and verifiable outputs.

## Inputs

- Dataset manifest: `ops/datasets/manifest.json`
- Fixture inventory: `ops/datasets/generated/fixture-inventory.json`
- Fixture policy: `ops/datasets/fixture-policy.json`

## Procedure

1. Validate fixture sources.

```bash
make ops-datasets-fetch
```

2. Run dataset quality checks.

```bash
make ops-dataset-qc
```

3. Run readiness checks before promotion.

```bash
make ops-readiness-scorecard
```

## Verify success

Expected result: dataset fetch, QC, and readiness checks pass with no contract violations.

## Rollback

If ingest validation fails, keep the current promoted dataset and fix source manifests before rerun.

## Next

- [Dataset Workflow](dataset-workflow.md)
- [Promotion Record](promotion-record.md)
