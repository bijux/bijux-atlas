# Audit Logging Model

Atlas writes audit events to the distinct `atlas_audit` log target when audit logging is enabled.

## What is recorded

- `config_loaded` during startup configuration resolution
- `startup` when the runtime begins serving
- `query_executed` for non-admin request completion
- `admin_action` for admin route access
- `ingest_started` and `ingest_completed` for control-plane ingest execution

## Timestamp policy

- Runtime events use `timestamp_policy = runtime-unix-seconds`
- Timestamps are real execution timestamps and should be excluded from deterministic file diffs

## What is not recorded

- Raw API keys
- Bearer tokens
- HMAC signatures
- Email-address style identifiers
- Direct client IP addresses in the canonical audit payload
