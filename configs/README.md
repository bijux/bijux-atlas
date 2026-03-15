# Configurations

`configs/` is the canonical home for repository configuration inputs.

Role layout:
- `configs/registry/` contains ownership, consumer, and inventory registries.
- `configs/schemas/` contains validation schemas and contract definitions.
- `configs/sources/` contains authored configuration inputs grouped by domain.
- `configs/examples/` contains non-authoritative example inputs.

The authoritative machine registries are `configs/registry/inventory/configs.json` and `configs/registry/contracts.json`.
Ownership and consumer mapping SSOT files:
- `configs/registry/owners.json`
- `configs/registry/consumers.json`

Common commands:
- `bijux dev atlas contracts configs --format table`
- `bijux dev atlas configs list --format json`

Example config files are allowed only under `configs/examples/`.
- Runtime server examples:
  - `configs/examples/runtime/server-minimal.toml`
  - `configs/examples/runtime/server-observability.toml`

Narrative docs belong under `docs/`, not under `configs/`.
