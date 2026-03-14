---
title: Crates you can depend on
audience: user
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - crates
related:
  - docs/reference/crates.md
  - release/crates-v0.1.toml
---

# Crates you can depend on

Use crates listed under `publish.allow` in `release/crates-v0.1.toml`.

- `bijux-atlas-core`: core identifiers, hashing, error and contract primitives.
- `bijux-atlas-model`: dataset/model structures and serialization contracts.
- `bijux-atlas-policies`: policy types and policy validation/evaluation APIs.
- `bijux-atlas-store`: storage contracts and backend adapters.
- `bijux-atlas-query`: query parser/planner/execution interfaces.
- `bijux-atlas-api`: API DTO and OpenAPI generation helpers.
- `bijux-atlas`: user-facing CLI integration surface.
- `bijux-atlas-ingest`: deterministic ingest workflow interfaces.
- `bijux-atlas-server`: runtime service crate and server integration points.

Do not depend on private crates listed in `publish.deny`.
