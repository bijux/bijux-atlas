# Logging Philosophy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define the invariant goals for production logging.

## Principles

1. Logs are structured first, text second.
2. Every operationally relevant event has a stable `event_name`.
3. Correlation identifiers (`request_id`, `trace_id`) are required for request-path events.
4. Sensitive values are never emitted in plaintext.
5. Sampling and rotation must remain deterministic and auditable.
