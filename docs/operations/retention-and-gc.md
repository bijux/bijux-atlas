# Retention and garbage collection

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: define retention policy and the safe garbage-collection procedure.

## Retention policy

- Keep all promoted datasets and pinned artifact hashes.
- Keep at least one previous release dataset for rollback.
- Delete only artifacts that are unreachable and unpinned.
- Never run destructive cleanup without a dry-run report.

Authoritative pins: `ops/inventory/pins.yaml`.

## Garbage-collection procedure

1. Generate plan:

```bash
cargo run -p bijux-atlas-cli -- atlas gc plan \
  --store-root artifacts/e2e-datasets \
  --catalog artifacts/e2e-datasets/catalog.json \
  --pins ops/inventory/pins.yaml
```

2. Review candidate deletions.
3. Apply cleanup only after approval:

```bash
cargo run -p bijux-atlas-cli -- atlas gc apply \
  --store-root artifacts/e2e-datasets \
  --catalog artifacts/e2e-datasets/catalog.json \
  --pins ops/inventory/pins.yaml \
  --confirm
```

## Verify success

Expected result: GC report shows only unreachable, unpinned deletions and no pinned losses.

## Rollback

If incorrect deletion is detected, restore from backup and republish catalog pointers.

## Next

- [Backup and Restore](release-workflow.md)
- [Dataset Workflow](dataset-workflow.md)
