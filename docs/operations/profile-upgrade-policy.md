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
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: define when governed install profiles may change and what proof is required.

Related ops contracts: `OPS-ROOT-023`, `OPS-LIFE-001`.

## Rules

- A profile may change only when `profiles.json`, the values overlay, and rollout-safety metadata stay aligned.
- A profile that changes runtime behavior must ship updated operator-facing documentation in the same change.
- Production-like profiles must keep digest pinning, filesystem hardening, and governed network policy posture intact.
- Lifecycle simulation for affected profiles must pass before the profile change can be treated as compatible.
