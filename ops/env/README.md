# ops/env

Environment overlays for ops deployment variants.

## Layout

- `ops/env/base/`: shared defaults for all environments.
- `ops/env/dev/`: local developer overrides.
- `ops/env/ci/`: CI execution overrides.
- `ops/env/prod/`: production-safe defaults and policy overlays.

## Overlay Contract

- Each environment directory must provide `overlay.json`.
- Overlay files are pure data (JSON); runtime logic is forbidden.
- Overlay merge order is fixed: `base -> <env>`.
- Merged values must include:
  - `namespace`
  - `cluster_profile`
  - `allow_write`
  - `allow_subprocess`
  - `network_mode`

Placeholder extension directories tracked with `.gitkeep`: `ops/env/base`, `ops/env/dev`, `ops/env/ci`, `ops/env/prod`, `ops/env/overlays`.

- Owner: `bijux-atlas-operations`
- Purpose: `environment overlay contracts for ops profiles`
- Consumers: `bijux dev atlas ops * --profile <name>, checks_ops_domain_contract_structure`
