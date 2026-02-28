# Diffs v1

- Owner: `bijux-atlas-query`

## What

Release-to-release gene diffs (`added`, `removed`, `changed`).

## Why

Cross-release comparison is a core adoption driver.

## Scope

Endpoints: `/v1/diff/genes`, `/v1/diff/region`.

## Non-goals

No transcript-level semantic diffing in v1.

## Contracts

- Changed means one of: coords, name, biotype, transcript_count.
- Diffs are computed from deterministic signature indexes.
- Explicit release/species/assembly required.

## Budgets

- Heavy class with strict limits and pagination (`limit` is row count per page).
- Cacheable responses when dataset pair hash is stable.

## Abuse controls

- Region span (`bp`) and limit caps (rows per page) are enforced by policy.
- Heavy-class bulkheads and overload shedding apply.

## Examples

```bash
$ curl -s "http://localhost:8080/v1/diff/genes?from_release=111&to_release=112&species=homo_sapiens&assembly=GRCh38&limit=25"
```

Expected output: JSON diff page with stable ordering and cursor.

## Failure modes

- Missing release pair => 400.
- Unsupported dataset pair => 404/422 policy rejection.
- Excessive query window or row limit => 400 rejection.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-server diff
```

Expected output: diff endpoint tests and golden snapshots pass.

## See also

- [Reference Index](../reference/index.md)
- [Dataset Operations Reference](../reference/dataset-operations.md)
- [SLO Targets](slo-targets.md)
