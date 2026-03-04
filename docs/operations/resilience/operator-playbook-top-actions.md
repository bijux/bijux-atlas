---
title: Operator playbook top actions
audience: operators
type: runbook
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-04
tags: [resilience, operations]
---

# Operator playbook top actions

1. Confirm failure code and impacted surface.
2. Freeze rollout or ingest when integrity risk is present.
3. Generate diagnose bundle.
4. Validate config and artifact chain before restart.
5. Apply runbook for the matching failure code.
6. Verify health and readiness after mitigation.
7. Re-run relevant failure scenario in evidence mode.
8. Attach evidence to incident record.
9. Execute rollback plan if mitigation fails.
10. Publish postmortem with corrective actions.
