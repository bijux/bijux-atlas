# Sequence v1

- Owner: `bijux-atlas-api`

## What

Read-only sequence retrieval by region and gene coordinates.

## Why

Clients need bounded sequence extraction without direct FASTA distribution.

## Scope

Endpoints: `/v1/sequence/region`, `/v1/genes/{gene_id}/sequence`.

## Non-goals

No translation, alignment, or variant projection.

## Contracts

- Strict region parsing and contig validation.
- Response size and max-bases limits enforced by policy.
- Optional metadata: GC% and masked fraction.

## Budgets

- Max bases per response enforced by policy.
- Sequence endpoints can require API key for high-cost ranges.

## Abuse controls

- Max bases per request.
- Optional API-key requirement for large ranges.
- Rate-limit and response-size guards.

## Examples

```bash
$ curl -s "http://localhost:8080/v1/sequence/region?release=112&species=homo_sapiens&assembly=GRCh38&region=1:100000-100120"
```

Expected output: JSON with sequence payload, bounded length, and dataset provenance headers.

## Failure modes

- Invalid region => 422 `InvalidQueryParameter`.
- Region too large => 413/422 policy rejection.
- Unknown contig => 400 rejection.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-server sequence
```

Expected output: boundary-condition tests pass.

## See also

- [Sequence Threat Model](sequence-threat-model.md)
- [Reference Index](../reference/index.md)
- [API Quick Reference](../api/quick-reference.md)
