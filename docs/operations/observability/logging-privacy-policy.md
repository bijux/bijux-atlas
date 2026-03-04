# Logging Privacy Policy

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define privacy constraints for observability logs.

## Requirements

- Avoid direct personal identifiers in runtime logs.
- Use stable technical identifiers (`request_id`, `query_id`, `dataset_id`) instead of raw user payloads.
- Keep retention aligned with `configs/observability/retention.yaml`.
