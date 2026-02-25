# Environment Overlays

- Owner: bijux-atlas-operations
- Stability: stable

## Canonical Files

- `ops/env/base/overlay.json`
- `ops/env/dev/overlay.json`
- `ops/env/ci/overlay.json`
- `ops/env/prod/overlay.json`

## Data-Only Rule

- Overlay files are JSON data only.
- Runtime logic is forbidden in `ops/env/`.
- Script-like file types under `ops/env/` are rejected by ops checks.

## Merge Contract

- Merge order is deterministic: `base -> <target-env>`.
- Target environments override base keys.
- Required keys after merge:
  - `namespace`
  - `cluster_profile`
  - `allow_write`
  - `allow_subprocess`
  - `network_mode`
