# Layout Domain Migration Map

Canonical check taxonomy is `atlasctl.checks.domains`.
Legacy `atlasctl.checks.layout` paths are migration sources and must not gain new checks.

## Domain Mapping

- `layout/architecture/*` -> `domains/repo` for runtime/module boundaries, `domains/internal` for self-validation checks.
- `layout/docs/*` -> `domains/docs`.
- `layout/makefiles/*` -> `domains/policies`.
- `layout/ops/*` -> `domains/ops`.
- `layout/root/*` -> `domains/repo`.
- `layout/workflows/*` -> `domains/repo` unless strictly ops-runtime lifecycle, then `domains/ops`.
- `layout/domains/*` -> mapped by intent into canonical domains:
  - artifacts/hygiene/orphans/root/public_surface -> `domains/repo`
  - contracts/policies -> `domains/policies`
  - scenarios/observability -> `domains/ops`

## Current Inventory

- `layout/architecture`: 26 python files
- `layout/docs`: 11 python files
- `layout/domains`: 37 python files
- `layout/makefiles`: 11 python files
- `layout/ops`: 53 python files
- `layout/product`: 13 python files
- `layout/root`: 7 python files
- `layout/scripts`: 7 python files
- `layout/workflows`: 8 python files

## Migration Rules

- Keep check IDs stable while relocating implementation modules.
- Runtime registry remains python-driven via `atlasctl.checks.registry`.
- New checks must be added only under `atlasctl.checks.domains`.
