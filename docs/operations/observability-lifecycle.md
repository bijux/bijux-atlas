# Observability Lifecycle

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@31b823e9454cbf8762982048a3d5f0ef9098c3f5`
- Reason to exist: define how observability assets change without breaking operators.

## Prereqs

- The dashboard source stays in `ops/observe/dashboards/atlas-observability-dashboard.json`.
- Alert rules stay in `ops/observe/alerts/atlas-alert-rules.yaml`.
- SLO definitions stay in `ops/observe/slo-definitions.json`.

## Install

- Update the governed source file and the matching schema or contract file together.
- Run `cargo run -q -p bijux-dev-atlas -- ops obs verify --allow-subprocess --allow-write --allow-network`.
- If release evidence is being produced, run `cargo run -q -p bijux-dev-atlas -- ops evidence collect --allow-subprocess --allow-write`.

## Verify

- Confirm `ops-obs-verify.json` reports `status: ok`.
- Confirm dashboard panels still satisfy `ops/observe/contracts/dashboard-panels-contract.json`.
- Confirm alert rules keep required labels from `configs/contracts/observability/label-policy.json`.

## Rollback

- Revert the dashboard, alert, or SLO source together with the matching schema change.
- Regenerate release evidence if the reverted files are already part of a candidate bundle.
