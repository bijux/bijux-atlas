# Make And Workflow Governance Entrypoints

This document inventories governance-oriented Make and workflow entrypoints during
the migration from `atlasctl` to `bijux dev atlas`.

## Canonical Governance Entrypoints

- `make dev-doctor`
- `make dev-check-ci`
- `make ops-doctor`
- `make ops-validate`
- `make ops-render`
- `make ops-status`
- `bijux dev atlas doctor --format json`
- `bijux dev atlas check run --suite ci_fast --format json`

## Wrapper Files Migrated To `bijux dev atlas`

- `makefiles/dev.mk`
- `makefiles/_ops.mk`
- `makefiles/ci.mk`
- `.github/workflows/atlas-dev-rust.yml`

## Remaining `atlasctl` References Outside Governance Wrappers

The repository still contains product, release, and compatibility workflows and
Make targets that use `atlasctl`. Those are not considered governance wrapper
entrypoints and must be migrated in domain-specific changes.

## Verification

- `bijux dev atlas check run --suite deep --include-internal --include-slow`
- `rg -n "atlasctl|scripts/areas" makefiles/_ops.mk makefiles/dev.mk makefiles/ci.mk .github/workflows/atlas-dev-rust.yml`
