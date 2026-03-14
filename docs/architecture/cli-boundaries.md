---
title: CLI Boundaries
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - cli
  - governance
---

# CLI boundaries

## User CLI (`bijux-atlas`)

User-facing product workflows only (catalog, dataset, ingest, query policy and related public actions).

## Developer CLI (`bijux-dev-atlas`)

Repository automation, governance, release validation, docs operations, and developer-only runtime diagnostics.

## Command ownership migration

`self-check` and `print-config-schema` were previously implemented in `crates/bijux-atlas/src/lib.rs` and dispatched in `crates/bijux-atlas/src/atlas_command_dispatch.rs`.

They now belong to developer tooling as:

- `bijux-dev-atlas runtime self-check`
- `bijux-dev-atlas runtime print-config-schema`
- `bijux-dev-atlas runtime explain-config-schema`
