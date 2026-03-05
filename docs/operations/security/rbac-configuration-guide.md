---
title: RBAC Configuration Guide
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# RBAC Configuration Guide

## Update sequence

1. update roles in `configs/security/roles.yaml`
2. update permissions in `configs/security/permissions.yaml`
3. update principal assignments in `configs/security/role-assignments.yaml`
4. validate policy and role references

## Validation commands

```bash
bijux-dev-atlas security authorization validate --format json
bijux-dev-atlas security authorization diagnostics --format json
```

## Contract checks

- `crates/bijux-dev-atlas/tests/security_rbac_policy_contracts.rs`
