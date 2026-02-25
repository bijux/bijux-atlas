# Ops Index

Human entry for the ops specification surface.

## Start Here

- Contract: `ops/CONTRACT.md`
- Control-plane ownership: `docs/development/tooling/dev-atlas-ops.md`
- Inventory map: `ops/docs/inventory-contracts.md`
- Migration window: `ops/docs/migration-window.md`
- Pin lifecycle: `ops/docs/pin-lifecycle.md`
- Generated index (compiled): `ops/_generated/INDEX.md`

## Domains

- `ops/stack/`
- `ops/k8s/`
- `ops/observe/` (target layout) and `ops/obs/` (migration compatibility)
- `ops/load/`
- `ops/e2e/`
- `ops/datasets/`
- `ops/report/`

## Make Surface

`makefiles/ops.mk` is delegation-only and routes to `bijux dev atlas ops ...`.

- `make ops-help` -> `bijux dev atlas ops --help`
- `make ops-doctor` -> `bijux dev atlas ops doctor --profile $(PROFILE) --format json`
- `make ops-validate` -> `bijux dev atlas ops validate --profile $(PROFILE) --format json`
- `make ops-render` -> `bijux dev atlas ops render --target helm --profile $(PROFILE) --allow-subprocess --format json`
- `make ops-install-plan` -> `bijux dev atlas ops install --kind --apply --plan --profile $(PROFILE) --allow-subprocess --allow-write --format json`
- `make ops-status` -> `bijux dev atlas ops status --target pods --profile $(PROFILE) --allow-subprocess --format json`
- `make ops-tools-verify` -> `bijux dev atlas ops verify-tools --allow-subprocess --format json`

## Generation Surface

Deterministic ops generation is owned by `bijux dev atlas ops generate ...`.

- `ops generate pins-index` writes a stable artifact index under `artifacts/atlas-dev/...`
- inventory inputs remain under `ops/` and are validated as SSOT by `ops doctor` / `ops validate`
- `ops stack` execution ownership stays in `bijux dev atlas ops`; make/workflows are wrappers only
