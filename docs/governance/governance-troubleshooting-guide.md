---
title: Governance Troubleshooting Guide
audience: operator
type: troubleshooting
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - troubleshooting
---

# Governance troubleshooting guide

## Rule enforcement drift

- Compare `configs/governance/enforcement/rules.json` and `ops/governance/enforcement/rules.snapshot.json`.
- Regenerate coverage references and rerun governance tests.

## Report mismatch

Run:

```bash
bijux-dev-atlas governance report --format json
```

Verify report path and summary fields against expected governance controls.
