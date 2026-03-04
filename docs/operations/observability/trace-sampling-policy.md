# Trace Sampling Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@f569762c0`
- Reason to exist: define deterministic sampling behavior for runtime tracing.

## Defaults

- Production default: ratio sampling via `ATLAS_TRACE_SAMPLING_RATIO`.
- Local diagnosis default: full sampling (`1.0`) for short windows only.
- Degraded mode default: reduce ratio to preserve service latency.

## Rules

1. Keep a stable sampling ratio during one incident window.
2. Change sampling only with a timestamped operator note.
3. Increase sampling before replaying a failing query class.
4. Decrease sampling if tracing causes measurable latency pressure.

## Verification

```bash
make ops-traces-check
```

Expected result: trace output remains correlated by request and route after sampling changes.
