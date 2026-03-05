---
title: Access Control Best Practices
audience: user
type: guide
stability: stable
owner: bijux-atlas-security
last_reviewed: 2026-03-05
---

# Access Control Best Practices

- keep `default_decision: deny` for all environments
- use least-privilege role definitions and explicit permission IDs
- separate human operator and automation principal roles
- validate authorization policy after each role or permission change
- treat role assignment changes as auditable governance updates

## Validation commands

```bash
bijux-dev-atlas security authorization validate --format json
bijux-dev-atlas security authorization diagnostics --format json
```
