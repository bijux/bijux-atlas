---
title: Authentication Architecture
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authentication Architecture

Atlas authentication is configured in `configs/security/auth-model.yaml` and runtime selection is controlled by `ATLAS_AUTH_MODE`.

## Supported modes

- `api-key`
- `token`
- `oidc`
- `mtls`

## Runtime enforcement

- middleware enforcement and identity extraction are implemented in `crates/bijux-atlas-server/src/runtime/request_utils.rs`
- startup and environment validation are implemented in `crates/bijux-atlas-server/src/config/mod.rs`

## Validation commands

```bash
bijux-dev-atlas security authentication diagnostics --format json
bijux-dev-atlas security authentication policy-validate --format json
```
