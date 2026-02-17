# Real Data E2E Regression Suite

Nightly-only realistic regression scenarios:

- `run_single_release.sh`: ingest + publish + serve + canonical snapshot verification.
- `run_two_release_diff.sh`: publish two releases and validate diff endpoints.
- `schema_evolution.sh`: old schema artifact remains queryable by current server.
- `upgrade_drill.sh`: upgrade under continuous probes, verify no request failures and stable semantics.
- `rollback_drill.sh`: rollback under continuous probes, verify health + semantic stability.

Runner:
- `run_all.sh`

Snapshots:
- Queries: `canonical_queries.json` (30 canonical GETs)
- Baseline: `snapshots/release110_snapshot.json`
- Generated: `artifacts/e2e/realdata/release110_snapshot.generated.json`
