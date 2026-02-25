# Stack Definition

- Owner: bijux-atlas-operations
- Stability: stable

## Canonical Sources

- Image pins: `ops/inventory/pins.yaml`
- Toolchain images: `ops/inventory/toolchain.json`
- Stack image mirror: `ops/stack/version-manifest.json`
- Profile inventory: `ops/stack/profiles.json`

## Validation

- `bijux dev atlas ops doctor --format json`
- `bijux dev atlas ops validate --format json`
- `bijux dev atlas ops stack plan --profile kind --format json`

## Generated Artifacts

- `ops/stack/generated/stack-index.json`
- `ops/stack/generated/dependency-graph.json`
- `ops/stack/generated/artifact-metadata.json`
