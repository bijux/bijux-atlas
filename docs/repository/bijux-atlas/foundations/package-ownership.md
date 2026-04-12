---
title: Package Ownership
audience: mixed
type: concept
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Package Ownership

Atlas documentation works better when ownership is explicit at package level.

`bijux-atlas` owns the product runtime: ingest, dataset state, query behavior,
runtime configuration, and user-facing contracts. `bijux-dev-atlas` owns the
repository control plane and maintainer automation. `bijux-atlas-ops` names the
operational surface shaped by `ops/`, `ops/k8s/`, `ops/stack/`, `ops/load/`,
and related evidence.

## Ownership Rule

- repository questions belong here when they explain the product package
- maintainer questions move to the maintainer handbook
- operations questions move to the operations handbook

## Why This Split Matters

Without the split, Atlas product behavior gets buried under Kubernetes,
workflows, and governance material that serves a different audience.

## Code Anchors

- `crates/bijux-atlas/`
- `crates/bijux-dev-atlas/`
- `ops/`
- `makes/`
