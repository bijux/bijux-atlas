# Logging Redaction Policy

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define safe-field expectations for runtime logs.

## Configuration

- `ATLAS_LOG_REDACTION_ENABLED` controls runtime redaction.
- Redaction must remain enabled in production environments.

## Minimum masking rules

- Secret-like keys (`secret`, `token`, `password`) must be masked.
- Payload fields not in safe-field policy must not be emitted as plaintext.
