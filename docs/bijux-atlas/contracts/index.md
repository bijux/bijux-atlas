---
title: Contracts
audience: mixed
type: index
status: canonical
owner: atlas-docs
last_reviewed: 2026-04-12
---

# Contracts

`bijux-atlas/contracts` is the section home for this handbook slice.

Use this section when the question is about what Atlas is intentionally trying
to keep stable for downstream users, operators, or automation consumers.

## What Belongs Here

- API compatibility promises
- runtime configuration and structured-output commitments
- plugin, artifact, and ownership boundaries
- the review rules maintainers should use before changing a documented surface

## Reading Rule

Read this section after you already understand the product model or workflow.
Contract pages are strongest when they sit on top of a clear mental model:

- start in [Foundations](../foundations/index.md) if the meaning of the surface is still unclear
- start in [Interfaces](../interfaces/index.md) if you first need to see the exact CLI, API, or config surface
- stay here when you need to decide whether a change is compatible, additive, or breaking

## Suggested Entry Points

- overall boundary and reading posture: [Contracts and Boundaries](contracts-and-boundaries.md)
- versioning and ownership expectations: [Ownership and Versioning](ownership-and-versioning.md)
- API-facing compatibility: [API Compatibility](api-compatibility.md)
- maintainer review discipline: [Compatibility Review Checklist](compatibility-review-checklist.md)

## Pages

- [API Compatibility](api-compatibility.md)
- [Artifact and Store Contracts](artifact-and-store-contracts.md)
- [Compatibility Review Checklist](compatibility-review-checklist.md)
- [Contract Reading Guide](contract-reading-guide.md)
- [Contracts and Boundaries](contracts-and-boundaries.md)
- [Operational Contracts](operational-contracts.md)
- [Ownership and Versioning](ownership-and-versioning.md)
- [Plugin Contracts](plugin-contracts.md)
- [Runtime Config Contracts](runtime-config-contracts.md)
- [Structured Output Contracts](structured-output-contracts.md)

## Exit Criteria

After using this section, a maintainer should be able to say:

- whether the surface is covered by a documented contract
- what kind of compatibility expectation applies
- which adjacent tests, docs, redirects, or release notes need to move with the change
