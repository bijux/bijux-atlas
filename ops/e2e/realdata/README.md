# Pointer

Canonical page: `docs/operations/e2e/realdata-drills.md`

## Canonical Runner

- `bijux dev atlas ops e2e run --suite realdata`
- `bijux dev atlas ops e2e run --suite realdata --fast`
- `bijux dev atlas ops e2e run --suite realdata --no-deploy`

Compatibility alias:
- `make ops-realdata`

## Core

- `make ops-help`
- `make ops-surface`
- `make ops-layout-lint`

## Topology

- `make ops-stack-up`
- `make ops-stack-down`
- `make ops-deploy`
- `make ops-undeploy`

## Domains

- `make ops-k8s-tests`
- `make ops-observability-validate`
- `make ops-load-smoke`
- `make ops-dataset-qc`
- `bijux dev atlas ops e2e run --suite realdata`

## Snapshot Policy

- Files under `ops/e2e/realdata/snapshots/` are example snapshots for contract verification and documentation.
- Required runtime fixtures are defined through `ops/e2e/fixtures/allowlist.json` and `ops/e2e/fixtures/fixtures.lock`.

## Full Flows

- `make ops-ref-grade-local`
- `make ops-full` to find the canonical entrypoint for this area.

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
