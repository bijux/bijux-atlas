# Debugging With Traces

1. Run `bijux-dev-atlas observe traces verify --format json`.
2. Confirm zero violations.
3. Review `artifacts/observe/trace-topology-diagram.mmd`.
4. Match broken requests to span attributes (`request_id`, `error_code`, `error_class`).
5. Validate request-to-log and request-to-metric correlation before root cause claims.
