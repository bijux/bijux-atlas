# Logging Best Practices

- Owner: `bijux-atlas-operations`
- Type: `guideline`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@053b86165`
- Reason to exist: lock stable conventions for structured logging quality and safety.

## Rules

1. Use stable `event_id` keys; do not rename lightly.
2. Include `request_id` on request-path logs.
3. Include `query_id` and `dataset_id` on query path logs.
4. Do not log secrets; rely on redaction policy.
5. Keep message text short and put detail in structured fields.
6. Use `warn` for retries/timeouts and `error` for hard failures.
