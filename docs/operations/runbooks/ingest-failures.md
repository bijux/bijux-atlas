# Ingest failures

1. Inspect ingest error counters and recent artifact changes.
2. Validate artifact integrity and schema compatibility.
3. Retry ingest after correcting source or configuration.
4. If repeated, switch to readonly serving mode and escalate.

## Evidence
- Required evidence bundle: ops/release/evidence/bundle.tar
- Contract reports: artifacts/ops/ops_run/observe/*.json
