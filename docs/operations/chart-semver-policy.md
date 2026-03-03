---
title: Chart Semver Policy
audience: operator
type: policy
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-03
---

# Chart Semver Policy

- Owner: `bijux-atlas-operations`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@3af24f78bdf0be1507efa8651298c45b68fa9e1e`
- Reason to exist: define when a chart change requires a major or minor version increment.

## Rules

- Use a major version bump when lifecycle compatibility checks can fail for existing releases.
- Use a major version bump when service identity, PVC definitions, ingress host shape, or required env keys change incompatibly.
- Use a minor version bump when the chart adds backward-compatible resources, fields, or optional capabilities.
- Use a patch version bump only when the change preserves existing values compatibility and rollout behavior.
