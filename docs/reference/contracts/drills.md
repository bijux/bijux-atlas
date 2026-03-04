# Drill Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the machine-readable contract for institutional drills.

## Contract IDs

- `DRILL-001`: every drill has a declared expected outcome and verification steps.
- `DRILL-002`: the registry covers baseline failure classes for Redis, network, and config rejection.

## Sources

- Drill registry: `ops/observe/drills.json`
- Drill registry schema: `ops/schema/drills/drills.schema.json`
- Drill report schema: `ops/schema/k8s/ops-drill.schema.json`

## Validation

Run `cargo run -q -p bijux-dev-atlas -- ops drills run --name warmup-pod-restart --allow-write --format json`.

The command emits a governed report and updates the drill summary report.
