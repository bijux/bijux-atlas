---
title: Authorization Metrics
audience: user
type: reference
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Authorization Metrics

Authorization metrics include role coverage, permission catalog size, and policy rule totals from diagnostics surfaces.

## Reports

- `ops/security/reports/access-control-metrics.example.json`

## Commands

```bash
bijux-dev-atlas security authorization diagnostics --format json
bijux-dev-atlas security authorization validate --format json
```
