---
title: ADR-0007 Tutorial Automation Domain Migration
audience: contributor
type: reference
stability: stable
owner: bijux-atlas-governance
last_reviewed: 2026-03-05
tags:
  - governance
  - adr
  - tutorials
---

# ADR-0007: Tutorial automation domain migration

- Status: accepted
- Date: 2026-03-05
- Owners: @atlas-governance

## Context

Tutorial execution and validation relied on `ops/tutorials/scripts` and Python test helpers, which fragmented governance and made automation ownership ambiguous.

## Decision

Tutorial automation is migrated into the `tutorials` domain in `bijux-dev-atlas`. Legacy tutorial script and Python-test automation paths are retired as migration reaches completion gates.

## Consequences

1. Tutorial workflows, verification, and reports are invoked from `bijux-dev-atlas tutorials ...`.
2. Tutorial artifacts remain in `ops/tutorials/`, while automation logic lives in Rust under `crates/bijux-dev-atlas`.

