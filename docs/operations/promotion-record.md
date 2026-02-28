# Promotion Record

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define what evidence is required for dataset promotion and rollback safety.

## Why you are reading this

Use this page to assemble and verify release evidence before promoting a dataset pointer.

## Required evidence

- Promotion rules: `ops/datasets/promotion-rules.json`
- Rollback policy: `ops/datasets/rollback-policy.json`
- Fixture drift report: `ops/_generated.example/fixture-drift-report.json`

## Procedure

1. Validate promotion prerequisites.

```bash
make ops-release-update
```

2. Confirm rollback path is available.

```bash
make ops-release-rollback
```

3. Record release decision and linked evidence artifacts.

## Verify success

Expected result: promotion record contains all required evidence and rollback route is verified.

## Rollback

Use `make ops-release-rollback` if post-promotion checks fail.

## Next

- [Release Workflow](release-workflow.md)
- [Incident Response](incident-response.md)
