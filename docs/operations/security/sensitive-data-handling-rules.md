# Sensitive Data Handling Rules

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define handling for sensitive fields and secrets.

## Rules

- Never log raw API keys, bearer tokens, HMAC signatures, or secret values.
- Emit only redacted fields for identity and network attributes.
- Reject artifacts that contain forbidden secret patterns in governed scans.

## Classification Source

- `configs/security/data-classification.yaml`
- `configs/security/data-protection.yaml`
