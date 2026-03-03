# Scenario Retention

- Owner: `bijux-atlas-performance`
- Purpose: document intentionally retained load scenarios that are not currently in suites.
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Unreferenced Scenario Retention

- `ops/load/scenarios/catalog-federated.json`: retained for federated catalog migration rehearsal.
- `ops/load/scenarios/large-dataset-simulation.json`: retained for large dataset stress baseline validation.
- `ops/load/scenarios/load-under-rollback.json`: retained for rollback failure recovery simulation.
- `ops/load/scenarios/load-under-rollout.json`: retained for rollout safety regression simulation.
- `ops/load/scenarios/multi-dataset-hotset.json`: retained for multi-dataset cache pressure analysis.
- `ops/load/scenarios/spike.json`: retained for transient overload signature comparison.
- `ops/load/scenarios/store-outage.json`: retained for storage outage blast-radius analysis.
