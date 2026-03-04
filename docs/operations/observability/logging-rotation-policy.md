# Logging Rotation Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: document runtime controls for log rotation and retention safety.

## Configuration

- `ATLAS_LOG_ROTATION_MAX_BYTES`
- `ATLAS_LOG_ROTATION_MAX_FILES`

Both values must be greater than zero.

## Operational guidance

- Keep rotation windows long enough for incident triage.
- Align rotation settings with retention and storage budget policies.
