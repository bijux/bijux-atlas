# Release Workflow Controls

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
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

- [Upgrade procedure](ops/release/upgrade-procedure.md)
- [Rollback procedure](ops/release/rollback-procedure.md)
- [Backup and restore validation](ops/release/backup-and-restore-validation.md)
