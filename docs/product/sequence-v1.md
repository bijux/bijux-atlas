# Sequence Endpoints v1

## Scope
- `/v1/sequence/region?region=seqid:start-end`
- `/v1/genes/{gene_id}/sequence` with optional `flank`
- Read-only FASTA random access via `.fai` offsets.

## Guardrails
- Enforced `max_sequence_bases`.
- Large responses require API key (`sequence_api_key_required_bases`).
- Per-IP sequence rate limit is separate from gene-query rate limits.
- Stable `ETag` based on dataset + canonical query.

## Optional Metadata
- `include_stats=1` adds:
  - `gc_fraction`
  - `masked_fraction`

## Non-Goals
- No translation.
- No alignment.
- No variant projection.
