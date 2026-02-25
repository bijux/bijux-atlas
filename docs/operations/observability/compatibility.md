# Observability Compatibility

- Owner: `bijux-atlas-operations`

## What

Defines the observability compatibility promise and contract evolution policy.

## Why

Keeps dashboards, alerts, and telemetry consumers stable across releases.

## Contracts

- Stable contract surfaces:
  - Metric names and label keys in `ops/observe/contracts/metrics-contract.json`
  - Span names in `docs/contracts/tracing.md`
  - Alert names in `ops/observe/contracts/alerts-contract.json`
  - Dashboard panel requirements in `ops/observe/contracts/dashboard-panels-contract.json`
- Additive-only policy for v1:
  - New metrics, labels, spans, and alerts may be added.
  - Existing names and label keys are not renamed or removed in v1.

## Breaking changes

- Breaking changes require:
  - Contract version bump.
  - Migration note in docs.
  - CI contract updates in the same change.
- Deprecated fields must remain present for one compatibility cycle before removal.

## Failure modes

Renamed telemetry fields break alert queries, dashboards, and external collectors.

## How to verify

```bash
make ops-observability-validate
make ssot-check
```
