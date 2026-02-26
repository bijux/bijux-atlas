# Release Workflows

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Update release

Use only make targets:

```bash
make ops-release-update DATASET=medium
```

Flow:

1. ingest + publish dataset
2. validate catalog
3. smoke check
4. promote catalog entry

## Roll back release pointer

Artifacts remain immutable; rollback only moves catalog pointer:

```bash
make ops-release-rollback DATASET=medium
```

## Notes

- `DATASET=medium|real1`
- Store root defaults to `artifacts/e2e-store` and can be overridden with `ATLAS_E2E_STORE_ROOT`.
