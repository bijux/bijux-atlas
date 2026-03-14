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

## Current workspace surfaces

- `bijux-atlas`: `serde` (default), `backend-local` (default), `backend-s3`, `bench-ingest-throughput`, `jemalloc`.
- `bijux-atlas-python`: `python-extension` for the optional `pyo3` native bridge.
- `bijux-dev-atlas`: `kind_integration` for private control-plane integration coverage.

Crates without declared features operate as always-on contract surfaces.
