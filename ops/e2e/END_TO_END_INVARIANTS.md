# End-to-End Invariants

- Canonical e2e suite and scenario identifiers must be declared in `ops/e2e/suites/suites.json` or `ops/e2e/scenarios/scenarios.json` before execution.
- Every e2e scenario in `ops/inventory/scenario-slo-map.json` must map to valid SLO ids, drill ids, and load suites.
- E2E expectation records must reference existing suite or scenario identifiers and must not invent hidden scenario ids.
- E2E fixture allowlists and fixture locks must remain hash-aligned with committed fixture inventories.
- Drill-linked operational scenarios must produce deterministic report artifacts with stable schema_version fields.
