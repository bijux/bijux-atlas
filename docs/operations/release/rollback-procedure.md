# Rollback Procedure

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define safe rollback execution and common pitfalls.

## Why you are reading this

Use this procedure when an upgrade or release causes service regression.

## Procedure

1. Confirm incident scope and affected surfaces.
2. Execute rollback.

```bash
make ops-release-rollback
```

3. Validate recovery.

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

## Pitfalls

- Rolling back without updated evidence can hide root cause.
- Rolling back while checks are still running can produce false positives.

## Verify success

Expected result: service health returns to baseline and paging alerts clear.

## Rollback

Do not attempt a second rollback action on top of an incomplete rollback. Stabilize on the last known good release before any further deploy.

## Next

- [Backup and Restore](backup-and-restore.md)
- [Incident Response](../incident-response.md)
