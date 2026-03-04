---
title: Why trust this
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - product
  - governance
related:
  - docs/product/how-this-repo-enforces-itself.md
  - docs/product/reliability-boundaries.md
---

# Why trust this

Atlas trust comes from enforced controls, not prose promises.

## Enforced controls

- Contract suites gate docs, configs, ops, and release evidence.
- Check suites fail fast on drift, orphan surfaces, and unpinned CI actions.
- Deterministic report artifacts are generated and validated against schemas.
- Governance metadata makes ownership and stability explicit.

## Evidence to review

- [How this repo enforces itself](how-this-repo-enforces-itself.md)
- [Reliability boundaries](reliability-boundaries.md)
- [For reviewers](for-reviewers.md)
- [Contract constitution](../contract.md)

## Decision rule

Trust the surface when:

1. Contract checks are green.
2. Generated evidence matches schemas.
3. Claimed behavior is documented in a canonical reference page.
