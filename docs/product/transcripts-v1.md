# Transcripts v1

- Owner: `bijux-atlas-query`

## What

Transcript summary retrieval and per-gene transcript listing.

## Why

Gene-level summaries need drill-down into transcript structures.

## Scope

Endpoints: `/v1/genes/{gene_id}/transcripts`, `/v1/trantooling/{tx_id}`.

## Non-goals

No canonical transcript selection policy in v1.

## Contracts

- Stable ordering with explicit tie-breakers.
- Pagination and limits enforced by policy.
- Parent gene validation follows ingest strictness policy.

## Budgets

- Classified as heavy query class for large transcript lists.
- Concurrency bulkheads apply.

## Abuse controls

- Limit bounds and cursor validation are mandatory.
- Query cost estimator can reject expensive combinations.

## Examples

```bash
$ curl -s "http://localhost:8080/v1/genes/ENSG00000139618/transcripts?release=112&species=homo_sapiens&assembly=GRCh38&limit=10"
```

Expected output: paginated transcript summaries with stable ordering and cursor.

## Failure modes

- Unknown gene => empty result or not-found endpoint semantics.
- Invalid cursor => 400 `InvalidCursor`.
- Policy limit exceeded => 400 rejection.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-query transcript
```

Expected output: transcript query plan and ordering tests pass.

## See also

- [Science Index](../science/index.md)
- [Reference Index](../reference/index.md)
- [API Quick Reference](../api/quick-reference.md)
