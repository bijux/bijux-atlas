# Runbook Generation From Control Graph

- Owner: `bijux-atlas-operations`
- Type: `workflow`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the canonical inputs and outputs for generated install, upgrade, and rollback operator guidance.

## Inputs

- `ops/inventory/control-graph.json`
- `ops/k8s/install-matrix.json`
- `ops/stack/profile-intent.json`
- `ops/inventory/toolchain.json`

## Outputs

- `artifacts/ops/<run_id>/generate/runbook.index.json`
- `docs/operations/`

## Invariants

- The control graph remains the upstream execution model for generated operator guidance.
- Install, upgrade, and rollback scenarios are derived from the governed install matrix, not hand-written duplicates.
- Profile intent metadata is attached when available so generated guidance explains allowed effects and dependencies.
- Tool references remain deterministic and come from the governed toolchain inventory.
