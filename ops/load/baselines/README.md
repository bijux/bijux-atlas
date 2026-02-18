# Load Baselines

- Owner: `bijux-atlas-operations`

Contains named baseline artifacts for perf regression comparison.

- `local.json`: baseline captured from local compose/kind environment.
- `ci-runner.json`: baseline captured from nightly CI runner.

Baseline updates require explicit approval in PR and should include rationale.

Baseline metadata should include:
- tool versions (`k6`, `kind`, `kubectl`, `helm`)
- machine profile (`os`, `arch`, `cpu`, `memory_gb`)

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
