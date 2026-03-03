# Institutional Drills

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@93ad533e5a4c4704f3a344db96b083570bb4d4b0`
- Reason to exist: provide the operator entrypoint for governed failure and recovery drills.

## Prereqs

- The governed drill registry is `ops/observe/drills.json`.
- The drill registry schema is `ops/schema/drills/drills.schema.json`.
- Drill reports are written under `artifacts/ops/<run_id>/reports/`.

## Install

- Choose a drill name from `ops/observe/drills.json`.
- Run `cargo run -q -p bijux-dev-atlas -- ops drills run --name <drill> --allow-write --format json`.

## Verify

- Confirm `ops-drill-<name>.json` exists.
- Confirm `ops-drills-summary.json` includes the run.
- Confirm the drill report `status` is `pass`.

## Rollback

- Revert any changed runtime or chart state before closing the drill.
- If the drill was used for a release candidate, regenerate release evidence so the bundle reflects the latest drill summary.

## Drill catalog

- `warmup-pod-restart`
- `redis-outage`
- `offline-network-deny`
- `catalog-unreachable`
- `store-unreachable`
- `offline-prewarm-serve`
- `rollout-failure-recovery`
- `invalid-config-rejected`
