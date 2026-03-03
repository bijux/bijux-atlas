# Privacy Stance

Atlas treats secrets, direct personal identifiers, and internal-only operational markers as
sensitive.

## Sensitive classes

- `pii`: direct identifiers such as `client_ip` and user email values
- `secrets`: tokens, API keys, HMAC signatures, and bearer credentials
- `proprietary`: internal operational markers that should not leak into public evidence

The canonical registry lives in `configs/security/data-classification.yaml`.

## Logging rule

Audit logs and standard logs may include only safe operational fields. Sensitive values must be
redacted or dropped before they are emitted.
