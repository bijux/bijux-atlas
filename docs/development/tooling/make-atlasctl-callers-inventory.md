# Makefile Atlasctl Callers Inventory

This inventory records Make targets and helper recipes that still invoke
`atlasctl` directly.

Purpose:
- make migration scope explicit
- separate governance wrapper migration from product/runtime migrations
- support review of `atlasctl` removal readiness

## Governance Wrapper Status

Migrated to `bijux dev atlas`:
- `makefiles/dev.mk`
- `makefiles/ops.mk`
- governance entrypoints in `makefiles/ci.mk` (`ci`, `ci-fast`, `ci-nightly`, `ci-docs`, `ci-dependency-lock-refresh`, `ci-help`)

Still `atlasctl` in `makefiles/ci.mk` (non-governance release/support wrappers):
- `ci-cosign-sign`
- `ci-cosign-verify`
- `ci-chart-package-release`
- `ci-init-tmp`

## Remaining Makefiles With Direct Atlasctl Invocations

- `makefiles/root.mk`
- `makefiles/docs.mk`
- `makefiles/product.mk`
- `makefiles/policies.mk`
- `makefiles/atlasctl.mk`
- `makefiles/_macros.mk`

## Removal Readiness Gate

`packages/atlasctl` and `makefiles/atlasctl.mk` cannot be deleted until:
- root/product/docs/policies wrappers are migrated
- workflows stop installing/running atlasctl for active lanes
- repo checks pass with zero governance `atlasctl` references
