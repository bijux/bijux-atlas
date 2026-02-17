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

- Alert rules are validated by `scripts/observability/check_alerts_contract.py`.
- Alerts must map to runbook drill procedures.

## Drill steps

- Trigger synthetic failure path.
- Verify alert fires and includes required labels.
- Execute linked runbook mitigation sequence.

## Failure modes

Invalid or stale alerts delay incident response.

## How to verify

```bash
$ python3 scripts/observability/check_alerts_contract.py
$ python3 scripts/observability/lint_runbooks.py
```

Expected output: alerts and runbook references pass checks.

## See also

- [Observability Index](INDEX.md)
- [Runbooks Index](../runbooks/INDEX.md)
- [Dashboard Contract](dashboard.md)
- `ops-ci`
