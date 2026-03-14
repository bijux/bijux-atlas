# Release Workflow

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `runbook`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define the stepwise release process for Atlas.

## Why you are reading this

Use this workflow when promoting a release into a target environment.

## Steps

1. Validate environment and prerequisites.

```bash
make ops-prereqs
```

2. Apply release update.

```bash
make ops-release-update
```

3. Validate runtime readiness and observability.

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

4. If checks fail, execute rollback.

```bash
make ops-release-rollback
```

## Verify success

Expected result: release update completes and all readiness and observability checks pass.

## Rollback

`make ops-release-rollback` restores the previous serving release pointer.

## Next

- [Release Operations](ops/release/index.md)
- [Upgrade Procedure](ops/release/upgrade-procedure.md)
