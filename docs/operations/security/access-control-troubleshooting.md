---
title: Access Control Troubleshooting
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Access Control Troubleshooting

## Symptoms

- authenticated request returns `403`
- expected role has no effect
- policy route rule does not match request path

## Diagnostic workflow

```bash
bijux-dev-atlas security authentication diagnostics --format json
bijux-dev-atlas security authorization diagnostics --format json
bijux-dev-atlas security policy-inspect --format json
bijux-dev-atlas security authorization validate --format json
```

## Common causes

- principal not mapped in `configs/security/role-assignments.yaml`
- permission action/resource mismatch
- route prefix in policy does not match actual endpoint
