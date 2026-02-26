# Dataset Promotion Pipeline

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

Deterministic promotion flow:

1. Ingest
- Run `atlas ingest` to produce artifacts.

2. Validate
- Run `atlas dataset validate` for checksums/schema/index gates.

3. Sign
- Sign release artifacts and container/image artifacts per release policy.

4. Publish dataset
- Run `atlas dataset publish` to publish immutable dataset payload.

5. Promote in catalog
- Run `atlas catalog promote` after dataset publish.

6. Update latest alias (optional)
- Run `atlas catalog latest-alias-update` only after successful promote.

7. Serve
- Atlas server refreshes catalog and serves released dataset.

8. Rollback
- If needed, run `atlas catalog rollback` (do not mutate published dataset files).

## See also

- `ops-ci`
