# Release Operations

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@0f088c31314aa61bc0ec69f1f5e683625b0d6bd5`
- Reason to exist: provide the release operator entrypoint.

## Why you are reading this

Use this section to ship, verify, and recover Atlas releases.

## Start here

- [Release operations](../release-operations.md)
- [Release Signing](../release-signing.md)
- [Release Workflow](../release-workflow.md)
- [Upgrade Procedure](upgrade-procedure.md)
- [Rollback Procedure](rollback-procedure.md)
- [Backup and Restore](backup-and-restore.md)
- [Backup and Restore Validation](backup-and-restore-validation.md)
- [Lane Guarantees](lane-guarantees.md)

## Verify success

```bash
make ops-prereqs
make ops-release-update
make ops-readiness-scorecard
```

Expected result: release checks pass and serving remains healthy.

## Next

- [Capacity Planning](capacity-planning.md)
- [Capacity planning worksheet](capacity-planning-worksheet.md)
- [Security Posture](../security-posture.md)
