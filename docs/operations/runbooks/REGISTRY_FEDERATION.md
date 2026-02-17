# Registry Federation Runbook

## Purpose

Operate Atlas with multiple artifact registries (primary + mirrors) while keeping dataset selection deterministic and safe.

## Configuration

- `ATLAS_REGISTRY_SOURCES`: comma-separated `name=scheme:value` entries.
- Supported schemes:
  - `local:/absolute/path`
  - `s3:https://bucket-or-gateway/path`
  - `http:https://readonly-registry/path`
- `ATLAS_REGISTRY_PRIORITY`: optional comma-separated ordered source names.
- `ATLAS_REGISTRY_TTL_MS`: source catalog refresh TTL.
- `ATLAS_REGISTRY_SIGNATURES`: optional `name=sha256(catalog.json)` pins.
- `ATLAS_REGISTRY_FREEZE_MODE`: when `true`, registry refresh is paused.

## Deterministic Merge Rules

- Catalogs are merged by configured priority order.
- First source wins for duplicate dataset IDs.
- Lower-priority duplicates are recorded as shadowed datasets.
- Final merged catalog is sorted by dataset canonical key.

## Fallback Rules

- Manifest/SQLite/FASTA fetch uses primary source for dataset when known.
- On failure, Atlas falls back to lower-priority sources.
- If all sources fail, request fails with stable store error.

## Health and Metrics

- Endpoint: `/debug/registry-health` (requires debug endpoints enabled).
- Metrics:
  - `bijux_registry_invalidation_events_total`
  - `bijux_registry_freeze_mode`

## Incident Procedure

1. Check `/debug/registry-health` for unhealthy source and shadowing patterns.
2. If primary is unstable, reorder with `ATLAS_REGISTRY_PRIORITY` and redeploy.
3. Pin trusted catalog digests with `ATLAS_REGISTRY_SIGNATURES`.
4. Enable `ATLAS_REGISTRY_FREEZE_MODE=true` to stop refresh churn during incident.
5. Keep serving cached datasets (`ATLAS_CACHED_ONLY_MODE=true`) if remote stores degrade.
