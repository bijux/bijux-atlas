---
title: Reliability boundaries
audience: user
type: concept
stability: stable
owner: platform
last_reviewed: 2026-03-04
tags:
  - product
  - reliability
related:
  - docs/product/production-ready-boundaries.md
  - docs/operations/index.md
---

# Reliability boundaries

This page states what Atlas validates today and what remains outside guarantee scope.

## Guaranteed when gates are green

- Documentation and contract surfaces are structurally coherent.
- CI workflow references are pinned and checked.
- Ops inventory and schema surfaces are validated by contracts.
- Docs/build outputs are deterministic and schema-validated.

## Explicitly outside current guarantees

- External service availability and network uptime.
- Third-party registry uptime and package distribution delays.
- Hardware-specific runtime variance outside documented budgets.

## Operator expectation

Use Atlas guarantees as repository and release-surface guarantees, then layer environment-specific SLOs in deployment policy.
