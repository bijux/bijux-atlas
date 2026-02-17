# Queries That Matter

- Owner: `bijux-atlas-product`

## What

Concrete institute questions mapped to API calls.

## Why

Product value is measured by answerable, reproducible queries.

## Scope

Examples use versioned `/v1` endpoints and explicit dataset dimensions.

## Non-goals

No synthetic toy flows without endpoint examples.

## Contracts

- Every query includes dataset identity or explicit alias endpoint.
- Pagination uses stable cursors.
- Errors return stable machine codes.

## Examples

```bash
$ curl -s "http://localhost:8080/v1/genes?release=112&species=homo_sapiens&assembly=GRCh38&name_prefix=BRCA&limit=5"
$ curl -s "http://localhost:8080/v1/genes/ENSG00000139618/transcripts?release=112&species=homo_sapiens&assembly=GRCh38&limit=10"
$ curl -s "http://localhost:8080/v1/diff/genes?from_release=111&to_release=112&species=homo_sapiens&assembly=GRCh38&limit=20"
```

Expected output: JSON objects with deterministic ordering, `data` arrays, and pagination metadata.

## Failure modes

- Unknown filter fields => 400 `InvalidQueryParameter`.
- Limit above policy => 400 `QueryRejectedByPolicy`.
- Invalid cursor signature => 400 `InvalidCursor`.

## How to verify

```bash
$ cargo run -p bijux-atlas-cli -- atlas smoke --dataset release=112,species=homo_sapiens,assembly=GRCh38
```

Expected output: smoke suite reports all canonical queries passed.

## See also

- [Sequence v1](sequence-v1.md)
- [Transcripts v1](transcripts-v1.md)
- [Diffs v1](diffs-v1.md)
