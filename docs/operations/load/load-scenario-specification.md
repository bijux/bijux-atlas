# Load Scenario Specification

Each load scenario must define:

- `id`: stable identifier
- `workload_kind`: `query`, `ingest`, or `mixed`
- `duration_secs`
- `target_rps`
- `ingest_ops_per_sec`
- `concurrency_profile`
- `success_thresholds`

Scenarios are deterministic and should reference pinned queries or deterministic generators.
