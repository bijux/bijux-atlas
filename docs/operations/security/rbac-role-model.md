---
title: RBAC Role Model
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# RBAC Role Model

RBAC roles define granted permissions and inheritance relationships.

## Inspection

```bash
bijux-dev-atlas security authorization roles --format json
```

## Assignment

```bash
bijux-dev-atlas security authorization assign --principal <principal> --role-id <role>
```
