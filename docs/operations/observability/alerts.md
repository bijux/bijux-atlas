# Alerts Contract

- Owner: `bijux-atlas-operations`

## What

Defines alert rules and operator drill steps.

## Why

Ensures alert behavior is actionable and reproducible.

## Scope

Alert rule definitions and validation scripts.

## Non-goals

Does not replace incident runbooks.

## Contracts

- Alert rules are validated by `crates/bijux-dev-atlas/src/observability/contracts/alerts/check_alerts_contract.py`.
- Alerts must map to runbook drill procedures.
- Alert rules carry version labels and contact pointers in `ops/observe/alerts/atlas-alert-rules.yaml`.

## Run drills

- Trigger a synthetic failure path.
- Verify alert firing and required labels.
- Execute linked runbook mitigation sequence.

## Failure modes

Invalid or stale alerts delay incident response.

## How to verify

```bash
$ make ops-alerts-validate
$ make observability-check
```

Expected output: alerts and runbook references pass checks.

## See also

- [Observability Index](INDEX.md)
- [Runbooks Index](../runbooks/INDEX.md)
- [Dashboard Contract](dashboard.md)
- `ops-ci`
