# E2E Realdata Drills

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Nightly realdata regression drills for schema evolution, upgrade, rollback, and diff semantics.

## Why

Validates behavior on realistic datasets without polluting PR CI runtime.

## Scope

`bijux dev atlas ops e2e run --suite realdata`, canonical query snapshots, and drift verification.

## Non-goals

Does not define fixture acquisition policy.

## Contracts

- Runner: `bijux dev atlas ops e2e run --suite realdata`
- Upgrade drill: `make ops-drill-upgrade`
- Rollback drill: `make ops-drill-rollback`

## Failure modes

Undetected regressions across releases or deployments.

## How to verify

```bash
$ bijux dev atlas ops e2e run --suite realdata
```

Expected output: realdata suite completes and snapshots verify.

## See also

- [E2E Index](INDEX.md)
- [Load Suites](../load/suites.md)
- [Dataset Promotion Pipeline](../dataset-promotion-pipeline.md)
- `ops-ci`
