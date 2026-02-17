# E2E Realdata Drills

- Owner: `bijux-atlas-operations`

## What

Nightly realdata regression drills for schema evolution, upgrade, rollback, and diff semantics.

## Why

Validates behavior on realistic datasets without polluting PR CI runtime.

## Scope

`ops/e2e/realdata/*.sh`, canonical query snapshots, and drift verification.

## Non-goals

Does not define fixture acquisition policy.

## Contracts

- Runner: `ops/e2e/realdata/run_all.sh`
- Upgrade drill: `ops/e2e/realdata/upgrade_drill.sh`
- Rollback drill: `ops/e2e/realdata/rollback_drill.sh`

## Failure modes

Undetected regressions across releases or deployments.

## How to verify

```bash
$ ./e2e/realdata/run_all.sh
```

Expected output: realdata suite completes and snapshots verify.

## See also

- [E2E Index](INDEX.md)
- [Load Suites](../load/suites.md)
- [Dataset Promotion Pipeline](../dataset-promotion-pipeline.md)
- `ops-ci`
