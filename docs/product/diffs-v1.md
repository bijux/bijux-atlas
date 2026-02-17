# Cross-Release Diffs v1

## Diff model
For the same `species + assembly`, Atlas computes per-gene release diffs:

- `added`: present in `to_release`, absent in `from_release`
- `removed`: present in `from_release`, absent in `to_release`
- `changed`: present in both, but gene signature differs

## What counts as changed
`changed` is defined by signature differences over:

- `name`
- `biotype`
- `seqid`
- `start`
- `end`
- `transcript_count`

## Endpoints
- `/v1/diff/genes`
- `/v1/diff/region`

Both endpoints are paginated with signed opaque cursors and deterministic ordering by `gene_id`.

## Latest alias policy
`latest` is allowed only when explicitly provided in `from_release` or `to_release`.
There is no implicit release default.

## Performance strategy
- Ingest writes `release_gene_index.json` (sorted `gene_id -> signature`).
- API performs merge-join over sorted index lists.
- Region diff filters are applied after merge-key reconciliation using stable coordinates.

## Caveats
- v1 diff is gene-level only.
- Transcript/exon-level diff semantics are out of scope.
