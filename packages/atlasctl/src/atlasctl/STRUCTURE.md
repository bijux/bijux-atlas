# Atlasctl Structure Contract

This file declares the intended top-level module layout for `packages/atlasctl/src/atlasctl`.

## Intended Top-Level Packages (max 10)

- `cli/`
- `commands/`
- `checks/`
- `core/`
- `contracts/`
- `registry/`
- `suite/`
- `reporting/`
- `policies/`
- `internal/`

## Migration Notes

- Legacy `docs/` top-level package remains transitional and should be folded into domain-owned modules.
- Check implementations are consolidating under `checks/<domain>/...`.
- Old deep paths under `checks/layout/.../checks/` are compatibility shims; canonical implementations live under:
  - `checks/make/impl/`
  - `checks/ops/impl/`
