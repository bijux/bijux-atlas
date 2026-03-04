# Token Authentication Flow

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define bearer token validation and request handling.

## Runtime contract

- Set `ATLAS_AUTH_MODE=token`
- Set `ATLAS_TOKEN_SIGNING_SECRET`
- Set `ATLAS_TOKEN_REQUIRED_ISSUER`
- Set `ATLAS_TOKEN_REQUIRED_AUDIENCE`
- Optionally set `ATLAS_TOKEN_REQUIRED_SCOPES`
- Optionally set `ATLAS_TOKEN_REVOKED_IDS`

## Validation pipeline

1. Parse `Authorization: Bearer <token>`.
2. Validate token structure and signature.
3. Validate `nbf` and `exp` windows.
4. Validate issuer and audience.
5. Validate required scopes.
6. Deny revoked token identifiers (`jti`).

## Failure semantics

Authentication failures return `401` with class `authentication` and classified reason codes such
as `token_expired`, `token_revoked`, or `token_scope_missing`.
