# Ops Datasets

## Purpose
Own dataset manifests, lifecycle policy, fixture governance, and generated inventory for operator-facing dataset workflows.

## Entry points
- `bijux-dev-atlas ops datasets fetch`
- `bijux-dev-atlas ops datasets validate`
- `bijux-dev-atlas ops datasets verify`
- `bijux-dev-atlas ops datasets qc diff`
- `bijux-dev-atlas ops datasets qc summary`
- `bijux-dev-atlas ops datasets lock`

## Contracts
- `ops/datasets/CONTRACT.md`
- `ops/datasets/manifest.lock`
- `ops/datasets/promotion-rules.json`
- `ops/datasets/consumer-list.json`
- `ops/datasets/freeze-policy.json`
- `ops/datasets/qc-metadata.json`
- `ops/datasets/fixture-policy.json`
- `ops/datasets/rollback-policy.json`

## Generated
- `ops/datasets/generated/dataset-index.json`
- `ops/datasets/generated/dataset-lineage.json`
- `ops/datasets/generated/fixture-inventory.json`

## Artifacts
- `artifacts/atlas-dev/ops/<run_id>/datasets/`

## Failure modes
- Dataset lock/schema mismatch.
- QC threshold failure.
- Promotion simulation contract regression.
- Fixture drift without manifest-lock evidence.

## Ownership
- Owner: `bijux-atlas-data`
- Consumers: `bijux-dev-atlas ops datasets ...`, `checks_ops_domain_contract_structure`, `checks_ops_fixture_governance`
