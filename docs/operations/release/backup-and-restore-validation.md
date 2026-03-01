# Backup and restore validation

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the checklist that proves backups and restore paths are usable before a real incident.

## Checklist

- [ ] registry and release metadata snapshot exists
- [ ] artifact pointers or immutable dataset roots are backed up
- [ ] restore instructions were exercised against a non-production target
- [ ] post-restore readiness and observability checks passed

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outputs:

- restored environment reaches readiness
- observability confirms the restored release is serving correctly

## Rollback

If restore validation fails, reapply the previous known-good snapshot and stop further destructive recovery steps.

## Next

- [Backup and restore](backup-and-restore.md)
- [Rollback procedure](rollback-procedure.md)
- [Release operations](../release-operations.md)
