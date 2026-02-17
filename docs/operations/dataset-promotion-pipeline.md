# Dataset Promotion Pipeline

Deterministic promotion flow:

1. Ingest
- Run `atlas ingest` to produce artifacts.

2. Validate
- Run `atlas dataset validate` for checksums/schema/index gates.

3. Sign
- Sign release artifacts and container/image artifacts per release policy.

4. Publish dataset
- Run `atlas dataset publish` to publish immutable dataset payload.

5. Publish catalog
- Run `atlas catalog publish` atomically after dataset publish.

6. Serve
- Atlas server refreshes catalog and serves released dataset.

7. Rollback
- If needed, run `atlas catalog rollback` (do not mutate published dataset files).

## See also

- `ops-ci`
