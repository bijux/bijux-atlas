# API Key Usage

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define operational handling of API keys for server-to-server access.

## Generate keys

Generate API keys with the security CLI command group and store only hashed entries in runtime
configuration.

## Configure runtime

- Set `ATLAS_AUTH_MODE=api-key`
- Set `ATLAS_ALLOWED_API_KEYS` to hashed key records
- Set `ATLAS_API_KEY_EXPIRATION_DAYS` and `ATLAS_API_KEY_ROTATION_OVERLAP_SECS`

## Rotation model

Use overlap windows to avoid downtime:

1. Add new key record with `not_before` set to rollout time.
2. Keep existing key record until all clients switch.
3. Mark old key record as `revoked=true`.

## Misuse response

Repeated invalid key attempts should trigger investigation using `authentication_failure_alert`
records and `atlas_policy_violations_total{policy="auth.api_key_invalid"}`.
