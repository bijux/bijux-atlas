---
title: Tutorial Reproducibility and Integrity Checks
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - reproducibility
  - integrity
related:
  - docs/operations/reproducibility-troubleshooting.md
  - docs/operations/security/security-troubleshooting-guide.md
---

# Tutorial Reproducibility and Integrity Checks

Run:

```bash
bijux-dev-atlas reproduce verify --format json
bijux-dev-atlas security validate --format json
```

These checks confirm deterministic outputs and integrity policies.
