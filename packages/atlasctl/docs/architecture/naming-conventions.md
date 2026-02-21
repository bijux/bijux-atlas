# Naming Conventions and Intent

This document defines naming intent in `atlasctl` to avoid ambiguous module roles.

## Module role names

- `command.py`: CLI entry shim only (`configure_*` + `run_*`).
- `contracts/`: JSON schemas, schema catalogs, and schema validation only.
- `internal/`: non-public commands and helpers; hidden from default help/listing.
- `registry.py`: canonical check registry module only (`checks/registry.py`).
- `runner.py`: canonical check runner module only (`checks/runner.py`).

## Preferred naming

- Use domain names instead of generic `layout/*` buckets for new checks:
  `repo_shape`, `makefiles`, `ops`, `docs`, `observability`, `artifacts`.
- Use `*_engine.py` or `*_handlers.py` for non-CLI execution plumbing.
- Keep wildcard imports/exports out of internal modules.

## API discipline

- Public API contract is documented in `docs/PUBLIC_API.md`.
- Public exports must be explicit via `__all__`.
- Wildcard exports are forbidden outside controlled public surfaces.
