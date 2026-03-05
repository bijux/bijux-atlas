---
title: Compare ops evidence bundles
owner: platform
stability: stable
last_reviewed: 2026-03-05
---

# Compare ops evidence bundles

Run:

```bash
bijux-dev-atlas ops evidence diff <bundle-a.tar> <bundle-b.tar> --format json
```

Review:
- `added`
- `removed`
- `changed`
- `high_risk_changed_paths`

High-risk paths include RBAC, NetworkPolicy, and Service-related artifacts.
