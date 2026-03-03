---
title: Rollback Limitations
audience: operator
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Rollback Limitations

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@3af24f78bdf0be1507efa8651298c45b68fa9e1e`
- Reason to exist: define what the kind-backed rollback simulation can and cannot guarantee.

## Limits

- Rollback can return to the previous Helm revision only when that revision still exists in release history.
- Rollback does not repair data migrations or application-level state changes that are not reversible.
- Rollback cannot guarantee safety if storage contracts or immutable service fields already drifted.
- Rollback is considered successful only when readiness, smoke checks, and service health all recover.
