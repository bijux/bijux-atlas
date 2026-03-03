---
title: Log Redaction Policy
audience: operator
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Log Redaction Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@5fcfe93aaeed218cb75ecb5c143ee3129fbe4bcf`
- Last changed: `2026-03-03`
- Reason to exist: define how logs are sanitized before they are allowed into release evidence.

## Required Redaction

- Replace values for `PASSWORD=`, `TOKEN=`, `SECRET=`, and `API_KEY=` with `[REDACTED]`.
- Replace bearer authorization values with `Authorization: Bearer [REDACTED]`.
- Reject the bundle if a redacted log still contains one of the governed secret markers.

## Scope

- This policy applies only to logs copied into `release/evidence/redacted-logs/`.
- Runtime source logs remain governed by operational access controls and retention rules outside the evidence bundle.
