---
title: Authorization Model
audience: contributor
type: concept
stability: stable
owner: architecture
last_reviewed: 2026-03-04
tags:
  - architecture
  - security
  - authorization
related:
  - docs/architecture/security/security-architecture.md
  - docs/operations/security/authorization-reference.md
---

# Authorization Model

## Authorization Philosophy

Atlas authorization is `deny-by-default`, deterministic, and auditable.
Every protected request is evaluated against role assignments, permission catalog entries,
and route policy rules before handler execution.

## RBAC Model

RBAC consists of four immutable contracts:

1. principal identity (`user`, `service-account`, `operator`, `ci`)
2. role assignment (principal to one or more roles)
3. permission catalog (action + resource-kind pairs)
4. route policy (principal, action, resource-kind, route prefix, effect)

## Permission Types

Permission units are action/resource pairs:

- `catalog.read` on `namespace`
- `dataset.read` on `dataset-id`
- `ops.admin` on `namespace`
- `dataset.ingest` on `project`

## Resource Model

Authorization resources are normalized to stable kinds:

- `dataset-id`: data-plane dataset requests
- `namespace`: operational namespace-level controls
- `project`: project-scoped ingest and lifecycle operations

## Administrative Permissions

Administrative paths require `ops.admin` and are scoped to
operator and controlled automation principals.

## Inheritance And Multi-Role Resolution

Role inheritance is additive. Effective permissions are the union of:

- direct role permissions
- inherited role permissions
- multiple assigned role permissions

Resolution is deterministic and does not include implicit wildcard grants.
