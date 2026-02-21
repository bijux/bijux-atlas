# Add a New Metric or Span

- Owner: `docs-governance`
- Stability: `stable`

## What

Contract workflow to add telemetry metrics and trace spans.

## Why

Keeps observability surfaces stable and queryable across releases.

## Scope

Metric/span registries, runtime instrumentation, docs generation, and checks.

## Non-goals

Does not prescribe dashboard layout details.

## Contracts

- Metrics registry: [Metrics Contract](../contracts/metrics.md).
- Trace spans registry: [Tracing Contract](../contracts/tracing.md).
- Generated docs and runtime constants must remain synchronized.

## Steps

1. Add entries to `METRICS.json` and/or `TRACE_SPANS.json`.
2. Add instrumentation in runtime code.
3. Update tests/alerting contracts if needed.
4. Regenerate contracts/docs.

## Failure modes

- Instrumentation without registry entry breaks contract drift gates.
- Registry entry without instrumentation creates dead telemetry.

## How to verify

```bash
$ make contracts
$ make test
$ make ops-observability-validate
```

Expected output: telemetry contract checks and observability gates pass.

## See also

- [Metrics Contract](../contracts/metrics.md)
- [Tracing Contract](../contracts/tracing.md)
- [Terms Glossary](../_style/terms-glossary.md)
