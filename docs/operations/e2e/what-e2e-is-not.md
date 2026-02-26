# What E2E Is Not

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Explicit negative contract for the e2e layer.

## Non-goals

- E2E is not an infrastructure patching surface.
- E2E is not a deployment fixup surface.
- E2E is not a place to run direct `helm upgrade/install` commands.
- E2E is not a place to run direct mutating `kubectl` repair actions.

## Contract

- Infra and deploy fixes must live in `ops/stack/*`, `ops/k8s/*`, or canonical `bijux dev atlas ops ...` entrypoints.
- E2E scripts should call `bijux dev atlas ops ...` entrypoints (or thin make wrappers) and only perform scenario validation logic.
