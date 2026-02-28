# Dataset Workflow

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: define canonical ingest and promotion flow for datasets.

## Workflow

1. Ingest dataset artifacts.
2. Validate artifact integrity.
3. Publish immutable dataset payload.
4. Promote catalog entry.
5. Verify query readiness.
6. Roll back catalog pointer if required.

## Verification

```bash
make ops-release-update DATASET=medium
```
