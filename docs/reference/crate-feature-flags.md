---
title: Crate feature flags
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - reference
  - crates
related:
  - docs/reference/crates.md
---

# Crate feature flags

## Policy

- Feature flags are allowed only for compile-time integration boundaries.
- Runtime semantics must not depend on optional feature toggles.

## Current publishable surfaces

- `bijux-atlas-core`: `serde` (default), deterministic core serialization surface.
- `bijux-atlas-ingest`: `bench-ingest-throughput` (benchmark-only).
- `bijux-atlas-store`: `backend-local` (default), `backend-s3`.
- `bijux-atlas-server`: `jemalloc`.

Crates without declared features operate as always-on contract surfaces.
