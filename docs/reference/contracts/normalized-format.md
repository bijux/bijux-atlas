# Normalized Ingest Format Contract

- Owner: `bijux-atlas-ingest`
- Stability: `evolving`

## What

Defines debug-only normalized ingest intermediate artifact format.

## Contracts

- SSOT schema: `docs/reference/contracts/schemas/NORMALIZED_FORMAT_SCHEMA.json`
- Encoded as `jsonl.zst`.
- Each record includes stable `record_id` and ordering fields.
- Schema versioned with additive-only evolution by default.
- Breaking changes require schema version bump.
- Not required for serving.

## How to verify

```bash
cargo test -p bijux-atlas-ingest normalized_replay_matches_db_content_counts -- --nocapture
```
