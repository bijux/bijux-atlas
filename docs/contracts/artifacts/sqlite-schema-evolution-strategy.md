# SQLite Schema Evolution Strategy

- Owner: `bijux-atlas-ingest`
- Stability: `stable`

## Strategy

- SQLite schema SSOT lives in `crates/bijux-atlas-ingest/sql/schema_v4.sql`.
- Schema upgrades are forward-only; downgrades are rejected.
- `schema_version` table is authoritative in DB.
- `atlas_meta.schema_version` is kept for compatibility reads.

## Version Bump Rules

- Any schema shape/index change must:
1. Increment `SQLITE_SCHEMA_VERSION`.
2. Update SSOT schema hash (`SQLITE_SCHEMA_SSOT_SHA256`).
3. Update schema drift digest test.
4. Add/adjust compatibility tests for forward upgrades.

## Compatibility Contract

- Additive changes are preferred.
- Existing columns/keys are not removed in the same major artifact stream.
- Old manifests remain parse-compatible via defaulted fields.

## Gates

- `schema_ssot_hash_is_stable`
- `schema_drift_gate_sqlite_master_digest_is_stable`
- `index_drift_gate_required_indexes_exist`
- `forward_only_upgrade_rejects_downgrade`
