# Ops Index

Use Make targets only.

## Core

- `make ops-help`
- `make ops-surface`
- `make ops-layout-lint`
- `make ops-check`
- `make ops-prereqs`
- `make ops-doctor`
- `make ops-lint`
- `make ops-fmt`
- `make ops-gen`
- `make ops-gen-check`
- `make ops-contracts-check`
- `make ops-e2e-validate`

## Topology

- `make ops-stack-up`
- `make ops-stack-down`
- `make ops-deploy`
- `make ops-undeploy`

## Domains

- `make ops-obs-up PROFILE=compose`
- `make ops-obs-verify`
- `make ops-obs-drill DRILL=prom-outage PROFILE=kind`
- `make ops-datasets-verify`
- `make ops-smoke`
- `make ops-e2e-smoke`
- `make ops-k8s-smoke`
- `make ops-k8s-suite`
- `make ops-load-suite SUITE=mixed-80-20`
- `make ops-local-reset`
- `make ops-k8s-tests`
- `make ops-observability-validate`
- `make ops-load-smoke`
- `make ops-dataset-qc`
- `make ops-realdata`

## Full Flows

- `make ops-ci-fast`
- `make ops-ci-nightly`
- `make ops-local-full`
- `make ops-ref-grade-local`
- `make ops-full`

## Area Contracts

- `ops/stack/CONTRACT.md`
- `ops/k8s/CONTRACT.md`
- `ops/obs/CONTRACT.md`
- `ops/load/CONTRACT.md`
- `ops/datasets/CONTRACT.md`
- `ops/e2e/CONTRACT.md`
- `ops/report/CONTRACT.md`
- `ops/fixtures/CONTRACT.md`
- `ops/run/CONTRACT.md`
