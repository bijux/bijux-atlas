# Authentication Troubleshooting

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide fast diagnostics for authentication failures.

## Check runtime mode

- Confirm `ATLAS_AUTH_MODE`
- Confirm matching credentials are configured for that mode

## Inspect failure telemetry

- Look for `authentication_context` and `authentication_failure_alert` logs
- Review `auth_policy_decision` entries for denied requests
- Review `atlas_policy_violations_total` by `policy` label

## Common failures

- `api_key_required`: missing `x-api-key`
- `api_key_invalid`: unknown, expired, or revoked key record
- `token_expired`: token `exp` passed
- `token_revoked`: token `jti` listed in `ATLAS_TOKEN_REVOKED_IDS`
- `proxy_identity_missing`: ingress header missing in `oidc` or `mtls` mode
