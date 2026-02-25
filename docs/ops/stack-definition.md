> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Stack Definition

- Owner: bijux-atlas-operations
- Stability: stable

## Canonical Sources

- Image pins: `ops/inventory/pins.yaml`
- Toolchain images: `ops/inventory/toolchain.json`
- Stack image mirror: `ops/stack/generated/version-manifest.json`
- Profile inventory: `ops/stack/profiles.json`

## Validation

- `bijux dev atlas ops doctor --format json`
- `bijux dev atlas ops check --format json`
- `bijux dev atlas ops stack check --profile kind --format json`

## Generated Artifacts

- `ops/stack/generated/stack-index.json`
- `ops/stack/generated/dependency-graph.json`
- `ops/stack/generated/artifact-metadata.json`
