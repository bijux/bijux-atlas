---
title: Authorization Model
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authorization Model

Atlas authorization uses deny-by-default policy evaluation with role-to-permission mapping.

## Sources

- role catalog: `configs/security/roles.yaml`
- permission catalog: `configs/security/permissions.yaml`
- policy rules: `configs/security/policy.yaml`
- role assignments: `configs/security/role-assignments.yaml`

## Validation

```bash
bijux-dev-atlas security authorization validate --format json
```
