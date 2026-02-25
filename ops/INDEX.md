# Ops Index

Human entry for the ops specification surface.

## Start Here

- Contract: `ops/CONTRACT.md`
- Control-plane ownership: `docs/development/tooling/dev-atlas-ops.md`
- Inventory map: `ops/report/docs/inventory-contracts.md`
- Migration window: `ops/report/docs/migration-window.md`
- Pin lifecycle: `ops/report/docs/pin-lifecycle.md`
- Generated index artifact: `ops/_generated.example/ops-index.json`
- Release evidence bundle: `ops/_generated.example/ops-evidence-bundle.json`

## Domains

- `ops/inventory/`
- `ops/schema/`
- `ops/env/`
- `ops/stack/`
- `ops/k8s/`
- `ops/observe/`
- `ops/load/`
- `ops/datasets/`
- `ops/e2e/`
- `ops/report/`
- `ops/_generated/`
- `ops/_generated.example/`
- Rename note: `ops/report/docs/observe-rename.md`

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
