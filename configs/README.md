# Configurations

`configs/` is the canonical home for repository configuration inputs.

Role layout:
- `configs/registry/` contains ownership, consumer, and inventory registries.
- `configs/schemas/` contains validation schemas and contract definitions.
- `configs/sources/` contains authored configuration inputs grouped by domain.
- `configs/examples/` contains non-authoritative example inputs.
- `configs/generated/` contains machine-written config artifacts that should be regenerated, not hand-edited.
- `configs/internal/` contains internal support material that is part of the repo contract but not a public config surface.

Machine-readable authorities:
- `configs/registry/inventory/configs.json` declares governed config groups and the file patterns they own.
- `configs/registry/owners.json` declares file-level and group-level ownership.
- `configs/registry/consumers.json` declares file-level consumer coverage.
- `configs/registry/schemas.json` declares file-level schema coverage.
- `configs/registry/contracts.json` declares the executable contracts that govern this tree.

Common commands:
- `bijux dev atlas contracts configs --format table`
- `bijux dev atlas configs list --format json`

Example config files are allowed only under `configs/examples/`.
- Runtime server examples:
  - `configs/examples/runtime/server-minimal.toml`
  - `configs/examples/runtime/server-observability.toml`
- Dataset examples:
  - `configs/examples/datasets/atlas-example-minimal`
  - `configs/examples/datasets/atlas-example-medium`
  - `configs/examples/datasets/atlas-example-large-synthetic`

Use the tree itself as the first signal:
- if the file is authored and operational, start in `configs/sources/`
- if the file explains ownership or coverage, start in `configs/registry/`
- if the file validates another config, start in `configs/schemas/`
- if the file is illustrative only, start in `configs/examples/`

Narrative product and maintainer documentation belongs under `docs/`, not under `configs/`.
