> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Config Pipeline Migration Cutoff

- Canonical config compiler pipeline target: `bijux-dev-atlas config validate|gen|diff|fmt`
- Authoritative inventory location: `ops/inventory/`
- Legacy location slated for deletion: `configs/inventory/` (if introduced during migration)

## Cutoff

- Cutoff date: `2026-05-01`

## Deletion Plan

1. Keep CI green on the dedicated config compiler lane (`validate + gen + diff --fail`).
2. Remove any legacy `configs/inventory/**` paths.
3. Keep repo guard `repo.config_migration_cutoff_legacy_locations` enabled after cutoff.
