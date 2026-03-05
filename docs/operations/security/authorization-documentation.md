---
title: Authorization Documentation
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authorization Documentation

Authorization is enforced by role and permission policy under deny-by-default semantics.

## Commands

```bash
bijux-dev-atlas security authorization roles --format json
bijux-dev-atlas security authorization permissions --format json
bijux-dev-atlas security authorization diagnostics --format json
bijux-dev-atlas security authorization validate --format json
bijux-dev-atlas security policy-inspect --format json
```

## Core references

- `docs/operations/security/authorization-model.md`
- `docs/operations/security/rbac-role-model.md`
- `docs/operations/security/permission-schema.md`
- `docs/operations/security/resource-access-policy-model.md`
