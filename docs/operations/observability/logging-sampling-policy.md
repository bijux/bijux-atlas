# Logging Sampling Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@053b86165`
- Reason to exist: define deterministic sampling behavior for high-volume logs.

## Configuration

- `ATLAS_LOG_SAMPLING_RATE` controls sampling ratio in `[0.0, 1.0]`.
- Sampling decision is deterministic per stable key.

## Guidance

- Keep `1.0` in incident diagnosis windows.
- Reduce ratio only when volume pressure is confirmed.
