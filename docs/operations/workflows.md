# Operational Workflows

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `runbook`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide canonical workflow definitions for ingest, validation, query readiness, and artifact promotion.

## Ingest workflow

1. Select approved source dataset inputs.
2. Run ingest pipeline with deterministic configuration.
3. Produce versioned artifacts and manifest output.
4. Capture evidence artifacts for traceability.

## Dataset preparation workflow

1. Normalize and stage source files.
2. Validate shape, schema, and required metadata.
3. Materialize partition and shard layout.
4. Freeze prepared dataset inputs for ingest.

## Dataset validation workflow

1. Execute schema and policy checks.
2. Verify checksums, manifests, and evidence completeness.
3. Confirm deterministic render and report outputs.
4. Mark dataset as eligible for promotion.

## Query workflow

1. Receive query request with validated parameters.
2. Build and execute deterministic query plan.
3. Return stable response schema and metadata.
4. Emit telemetry and request evidence fields.

## Artifact promotion workflow

1. Select validated artifact version.
2. Verify policy and readiness gates.
3. Promote catalog pointer to the selected artifact.
4. Verify runtime health and rollback readiness.

## Related runbooks

- [Dataset Workflow](dataset-workflow.md)
- [Fixture Dataset Ingest](fixture-dataset-ingest.md)
- [Release Workflow](release-workflow.md)
- [Promotion Record](promotion-record.md)
