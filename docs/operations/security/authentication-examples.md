# Authentication Examples

- Owner: `bijux-atlas-security`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide concrete auth configuration examples.

## API key mode

```bash
ATLAS_AUTH_MODE=api-key
ATLAS_ALLOWED_API_KEYS='hash=<sha256>|expires=4102444800'
ATLAS_API_KEY_EXPIRATION_DAYS=90
ATLAS_API_KEY_ROTATION_OVERLAP_SECS=86400
```

## Token mode

```bash
ATLAS_AUTH_MODE=token
ATLAS_TOKEN_SIGNING_SECRET='replace-with-strong-secret'
ATLAS_TOKEN_REQUIRED_ISSUER='atlas-auth'
ATLAS_TOKEN_REQUIRED_AUDIENCE='atlas-api'
ATLAS_TOKEN_REQUIRED_SCOPES='dataset.read'
```

## OIDC boundary mode

```bash
ATLAS_AUTH_MODE=oidc
# ingress must set x-forwarded-user or x-atlas-oidc-subject
```
