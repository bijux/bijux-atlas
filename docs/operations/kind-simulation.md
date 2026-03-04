---
title: Kind Simulation
audience: operator
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Kind Simulation

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define the deterministic install simulation surface for kind-backed validation.

## Prereqs

- Install `kind`, `kubectl`, `helm`, and `kubeconform`.
- Use the generated kind cluster config in `ops/k8s/kind/cluster.yaml`.
- Run commands with `--allow-subprocess --allow-write --allow-network`.

## Install

```bash
make kind-up
bijux-dev-atlas ops helm install --profile profile-baseline --cluster kind --allow-subprocess --allow-write --allow-network --format json
bijux-dev-atlas ops smoke --profile profile-baseline --cluster kind --allow-subprocess --allow-write --allow-network --format json
```

## Verify

- `ops-install.json` records helm output, readiness wait, kubeconform output, configmap env keys, runtime allowlist status, profile intent, and profile metadata.
- `ops-smoke.json` records `/healthz`, `/readyz`, and `/v1/version` status, latency, and body hash.
- `ops-simulation-summary.json` combines install, smoke, and cleanup evidence per profile for the current run.

## Failure Triage

- Install report failures usually mean helm render, apply, or readiness wait failed.
- Smoke report failures mean the service answered but the health or version routes did not return `200`.
- Cleanup report failures mean uninstall completed but namespace resources still exist.

## Rollback

- Run `bijux-dev-atlas ops helm uninstall --profile <name> --cluster kind --allow-subprocess --allow-write --allow-network --format json`.
- If the cluster itself is unhealthy, run `make kind-reset`.
