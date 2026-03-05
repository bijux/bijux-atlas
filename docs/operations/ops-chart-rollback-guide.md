# Ops Chart Rollback Guide

1. Identify last known-good chart digest.
2. Roll back Helm release to previous revision.
3. Re-run install validation and smoke checks.
4. Attach rollback evidence bundle to incident record.
