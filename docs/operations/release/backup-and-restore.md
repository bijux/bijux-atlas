# Backup and Restore

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define backup and restore expectations for registry and artifact stores.

## Why you are reading this

Use this procedure before risky release actions and during recovery from data loss.

## Backup scope

- Dataset registry and promotion metadata.
- Artifact store roots used by serving and e2e paths.

## Procedure

1. Snapshot registry and artifact pointers before release.
2. Store snapshot in approved backup location.
3. For restore, rehydrate snapshot and rerun readiness checks.

## Verify success

```bash
make ops-readiness-scorecard
```

Expected result: restored pointers resolve and service reads valid artifacts.

## Rollback

If restore validation fails, reapply last known good snapshot and rerun readiness checks.

## Next

- [Release Workflow](../release-workflow.md)
- [Retention and garbage collection](../retention-and-gc.md)
