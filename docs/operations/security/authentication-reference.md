---
title: Authentication Documentation
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authentication Documentation

Authentication surfaces for Atlas are documented and validated through stable CLI commands.

## Commands

```bash
bijux-dev-atlas security authentication api-keys --format json
bijux-dev-atlas security authentication token-inspect --token <jwt> --format json
bijux-dev-atlas security authentication diagnostics --format json
bijux-dev-atlas security authentication policy-validate --format json
```

## Core references

- `docs/operations/security/authentication-architecture.md`
- `docs/operations/security/authentication-protocol-options.md`
- `docs/operations/security/api-key-authentication-support.md`
- `docs/operations/security/token-authentication-strategy.md`
