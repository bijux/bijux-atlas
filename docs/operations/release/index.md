# Release Operations

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: provide the release operator entrypoint.

## Why you are reading this

Use this section to ship, verify, and recover Atlas releases.

## Start here

- [Release Workflow](../release-workflow.md)
- [Upgrade Procedure](upgrade-procedure.md)
- [Rollback Procedure](rollback-procedure.md)
- [Backup and Restore](backup-and-restore.md)

## Verify success

```bash
make ops-release-update
make ops-readiness-scorecard
```

Expected result: release checks pass and serving remains healthy.

## Next

- [Capacity Planning](capacity-planning.md)
- [Security Posture](../security-posture.md)
