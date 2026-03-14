# Rollback procedure

1. Confirm rollback trigger from alert and release gate failure.
2. Execute rollback to last known good release.
3. Verify service health, SLO, and alert recovery.
4. Attach evidence bundle and readiness report to the incident.

## Evidence
- Required evidence bundle: ops/release/evidence/bundle.tar
- Contract reports: artifacts/ops/ops_run/observe/*.json
