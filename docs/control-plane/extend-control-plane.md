---
title: Extend the Control-plane
audience: contributor
type: guide
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - control-plane
  - extensibility
related:
  - docs/control-plane/index.md
  - docs/development/index.md
verification: true
---

# Extend the Control-plane

This is the canonical extension guide for adding control-plane behavior.

## Add a check

1. Define check intent and output contract.
2. Add the check implementation and registry wiring.
3. Add targeted tests and run the relevant suite.
4. Update docs and generated artifacts.

Detailed walkthrough:
- [Add a check in 30 minutes](add-a-check-in-30-minutes.md)

## Add a contract registry

1. Define the contract scope and schema authority.
2. Add registry entries and runner wiring.
3. Add snapshot/integration tests for determinism.
4. Update documentation and examples.

Detailed walkthrough:
- [Add a contract registry in 30 minutes](add-a-contract-registry-in-30-minutes.md)

## Change safety checklist

- Keep command names stable and explicit.
- Keep generated outputs deterministic.
- Keep docs references canonical and avoid duplicate narrative pages.
- Run control-plane and docs gates before merging.
