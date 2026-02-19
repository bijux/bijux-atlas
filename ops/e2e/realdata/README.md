# Pointer

Canonical page: `docs/operations/e2e/realdata-drills.md`

## Canonical Runner

- `./ops/run/e2e.sh --suite realdata`
- `./ops/run/e2e.sh --suite realdata --fast`
- `./ops/run/e2e.sh --suite realdata --no-deploy`

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
- `./ops/run/e2e.sh --suite realdata`

## Full Flows

- `make ops-ref-grade-local`
- `make ops-full` to find the canonical entrypoint for this area.

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
