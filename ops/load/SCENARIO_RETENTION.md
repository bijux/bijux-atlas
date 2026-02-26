# Load Scenario Retention

- Owner: `bijux-atlas-operations`
- Purpose: `declare retained load scenarios that are intentionally not referenced by ops/load/suites/suites.json`
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Unreferenced Scenario Retention

- `ops/load/scenarios/catalog-federated.json`: reserved for catalog federation readiness validation; suite wiring is pending runtime branch enablement.
- `ops/load/scenarios/large-dataset-simulation.json`: retained for partial dataset and large dataset portability/deletion-risk simulations.
- `ops/load/scenarios/load-under-rollback.json`: script-driven rollout/rollback lane uses `runner`; scenario file is retained as documented input profile.
- `ops/load/scenarios/load-under-rollout.json`: script-driven rollout lane uses `runner`; scenario file is retained as documented input profile.
- `ops/load/scenarios/multi-dataset-hotset.json`: retained for future hotset/cache-topology regression coverage.
- `ops/load/scenarios/spike.json`: retained as baseline spike profile distinct from `spike-overload-proof`.
- `ops/load/scenarios/store-outage.json`: retained as baseline outage profile distinct from `store-outage-under-spike`.

## Deletion Rule

- If a scenario remains unreferenced by `ops/load/suites/suites.json`, it must appear in the retention list above with an explicit reason.
