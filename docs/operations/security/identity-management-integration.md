---
title: Identity Management Integration
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Identity Management Integration

Atlas integrates with institutional identity systems at trusted boundaries.

## Integration model

- OIDC and mTLS identities are validated at ingress or mesh boundary
- approved identity headers are forwarded to Atlas
- request identity context is normalized before authorization checks

## Operational guidance

- deploy behind approved auth proxy: `docs/operations/security/deploy-behind-auth-proxy.md`
- validate runtime mode and identity source using diagnostics commands
