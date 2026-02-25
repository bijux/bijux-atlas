# Schema Reference Allowlist

Schemas that are intentionally not referenced directly by current runtime artifacts.

- `ops/schema/e2e-realdata-scenarios.schema.json`: compatibility schema retained for realdata snapshot migration.
- `ops/schema/e2e-scenarios-unified.schema.json`: compatibility schema retained for unified scenario rollout.
- `ops/schema/load/query-pack-catalog.schema.json`: reserved for query catalog publication workflow.
- `ops/schema/meta/budgets.schema.json`: governance budget schema retained for policy extensions.
- `ops/schema/meta/ports.schema.json`: governance port schema retained for future inventory validation.
- `ops/schema/datasets/corruption-drill-report.schema.json`: reserved for corruption drill report publication.
- `ops/schema/datasets/fixture-policy.schema.json`: compatibility schema retained for fixture policy evolution.
- `ops/schema/datasets/promotion-report.schema.json`: reserved for dataset promotion reporting.
- `ops/schema/datasets/qc-summary.schema.json`: reserved for dataset QC summary outputs.
- `ops/schema/k8s/conformance-report.schema.json`: reserved for conformance report publication.
- `ops/schema/load/k6-suite.schema.json`: reserved for suite-level publication payloads.
- `ops/schema/load/perf-baseline.schema.json`: reserved for baseline exchange payloads.
- `ops/schema/load/pinned-queries-lock.schema.json`: reserved for pinned query lock publication.
- `ops/schema/configs/public-surface.schema.json`: inventory surface schema retained under schema locality contract.
- `ops/schema/inventory/contracts-map.schema.json`: authored inventory map schema validated by dedicated inventory contract checks.
- `ops/schema/observe/budgets.schema.json`: reserved for observability budget policies.
- `ops/schema/observe/drill-manifest.schema.json`: compatibility schema retained while inventory drills use inventory-local schema.
- `ops/schema/meta/artifact-allowlist.schema.json`: retained for compatibility wrappers and report policy references.
- `ops/schema/meta/namespaces.schema.json`: retained for compatibility wrappers and governance policy history.
- `ops/schema/meta/ownership.schema.json`: retained for compatibility wrappers and governance policy history.
- `ops/schema/meta/pins.schema.json`: retained for compatibility wrappers and governance policy history.
- `ops/schema/report/lane.schema.json`: reserved for lane-scoped report fragments.
- `ops/schema/report/stack-contract.schema.json`: reserved for stack contract report fragments.
- `ops/schema/report/stack-health-report.schema.json`: reserved for stack health snapshots.
- `ops/schema/report/stack-ports-inventory.schema.json`: reserved for stack port inventory snapshots.
- `ops/schema/report/unified.schema.json`: reserved for unified report payload publication.
