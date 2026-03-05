---
title: Multi-Tenant Access Boundaries
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Multi-Tenant Access Boundaries

Atlas supports tenant boundary modeling through governed resource kinds and policy rules.

## Boundary controls

- tenant resource kind in `configs/security/resources.yaml`
- principal-to-role mapping in `configs/security/role-assignments.yaml`
- policy rule resource filters in `configs/security/policy.yaml`

## Guidance

Restrict administrative roles to tenant-scoped resources unless explicit cross-tenant operation is required.
