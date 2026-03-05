---
title: Token Authentication Strategy
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Token Authentication Strategy

Token authentication is enabled with `ATLAS_AUTH_MODE=token`.

## Strategy

- require signed bearer token claims
- validate issuer, audience, expiry, and token identity where configured
- enforce scope and policy binding before authorization

## Diagnostic command

```bash
bijux-dev-atlas security authentication token-inspect --token <jwt> --format json
```
