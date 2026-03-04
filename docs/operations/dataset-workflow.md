# Dataset Workflow

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `runbook`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the canonical dataset lifecycle used by operators.

## Lifecycle

1. Select pinned fixture or source dataset manifest.
2. Ingest artifacts using approved ingest path.
3. Validate checksum, schema, and readiness checks.
4. Publish immutable dataset artifacts.
5. Promote dataset pointer in catalog.
6. Verify query readiness and observability signals.

## Verify success

```bash
make ops-release-update
make ops-readiness-scorecard
```

Expected result: promoted dataset serves successfully and readiness checks pass.

## Rollback

Repoint catalog to last known good dataset and re-run readiness checks.

## Next

- [Fixture Dataset Ingest](fixture-dataset-ingest.md)
- [Promotion Record](promotion-record.md)
- [Retention and garbage collection](retention-and-gc.md)
