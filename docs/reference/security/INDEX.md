# Security Reference Index

- Owner: `bijux-atlas-security`

## What

Reference entrypoint for runtime and API security controls.

## Why

Security posture must be explicit and testable.

## Scope

Threat model, limits, SSRF controls, optional auth/HMAC modes.

## Non-goals

No claims beyond implemented controls.

## Contracts

- [Threat Model](threat-model.md)
- [Limits](limits.md)
- [SSRF Controls](ssrf.md)
- [Auth and HMAC Options](auth-hmac.md)

## Failure modes

Missing control documentation can create unsafe deployment defaults.

## How to verify

```bash
$ make audit
```

Expected output: security checks pass with no errors.

## See also

- [Incident Response](../../operations/incident-response.md)
- [Store Reference](../store/INDEX.md)
- [Contracts Errors](../../contracts/errors.md)
