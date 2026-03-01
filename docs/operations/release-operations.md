# Release operations

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: provide the single operator playbook for upgrade, rollback, and backup coordination.

## Scope

- pre-release prerequisites
- upgrade execution
- rollback execution
- backup and restore readiness

## Workflow

1. confirm prerequisites and backup readiness
2. run the release update
3. verify readiness, observability, and smoke coverage
4. roll back immediately if verification fails

## Verify success

```bash
make ops-prereqs
make ops-release-update
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outputs:

- prerequisites pass
- release update completes
- readiness and observability checks return healthy status

## Rollback

```bash
make ops-release-rollback
```

## Next

- [Upgrade procedure](release/upgrade-procedure.md)
- [Rollback procedure](release/rollback-procedure.md)
- [Backup and restore validation](release/backup-and-restore-validation.md)
