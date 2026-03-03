---
title: Profile Upgrade Policy
audience: operator
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Profile Upgrade Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@3af24f78bdf0be1507efa8651298c45b68fa9e1e`
- Reason to exist: define when governed install profiles may change and what proof is required.

## Rules

- A profile may change only when `profiles.json`, the values overlay, and rollout-safety metadata stay aligned.
- A profile that changes runtime behavior must ship updated operator-facing documentation in the same change.
- Production-like profiles must keep digest pinning, filesystem hardening, and governed network policy posture intact.
- Lifecycle simulation for affected profiles must pass before the profile change can be treated as compatible.
