# Ops Layout Migration Window

- Start: 2026-02-23
- Cutoff (legacy paths forbidden): 2026-04-01

## Scope

Legacy paths scheduled for removal after cutoff:

- `ops/schema/**`
- `ops/_meta/{ownership,surface,contracts,layer-contract}.json`
- `ops/inventory/gc-pins.json`
- `ops/stack/version-manifest.json`
- `ops/stack/versions.json`
- `ops/_generated.example/`

## Deletion Plan

1. Keep canonical files updated under `ops/schema/` and `ops/inventory/`.
2. Run duplicate-inventory checks to prevent drift during transition.
3. Land runtime/CLI cutover to canonical paths.
4. Remove legacy files and enable legacy-path hard guard.
