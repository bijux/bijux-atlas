# Fixture Taxonomy

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define fixture classes and constraints for E2E reliability.

## Fixture classes

- `baseline`: minimal dataset for smoke and startup checks.
- `regression`: targeted datasets for known failure classes.
- `stress`: high-cardinality datasets used by load and soak checks.

## Constraints

- Fixtures must be pinned and checksum-verified.
- E2E flows must use fixture IDs, not ad-hoc paths.
- Fixture changes require updated promotion evidence.

Authoritative sources:

- `ops/datasets/manifest.json`
- `ops/datasets/generated/fixture-inventory.json`

## Verify success

```bash
make ops-datasets-fetch
```

Expected result: fixture fetch and checksum validation pass.

## Next

- [Dataset Workflow](../dataset-workflow.md)
- [Fixture Dataset Ingest](../fixture-dataset-ingest.md)
