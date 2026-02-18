# Grafana Dashboard Source (Stub)

- Owner: `bijux-atlas-operations`

Canonical dashboard documentation: `docs/operations/observability/dashboard.md`.

## Make Target

Use \# Ops Index

Use Make targets only.

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
- `make ops-realdata`

## Full Flows

- `make ops-ref-grade-local`
- `make ops-full` to find the canonical entrypoint for this area.
