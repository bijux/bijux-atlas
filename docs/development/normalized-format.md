# Normalized Format (Debug)

- Owner: `bijux-atlas-ingest`
- Stability: `evolving`

## What

Optional debug artifact: `normalized_features.jsonl.zst` under dataset `derived/`.

## Why

Gives an auditable, deterministic, replayable intermediate view of ingest feature graph.

## How

- Enable with `atlas ingest --emit-normalized-debug`.
- Replay counts with `atlas ingest-replay --normalized <path>`.
- Diff releases with `atlas ingest-normalized-diff --base <a> --target <b>`.

## Contracts

- Schema SSOT: `docs/contracts/NORMALIZED_FORMAT_SCHEMA.json`.
- Stable ordering: `kind, seqid, start, end, record_id`.
- Never required by serving path.

## Policy gate

- `--prod-mode` + `--emit-normalized-debug` is rejected.
- Normalized output is debug-only and can be disabled in production workflows.
