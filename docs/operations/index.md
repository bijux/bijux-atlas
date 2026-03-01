---
title: Operations
audience: operator
type: concept
stability: stable
owner: bijux-atlas-operations
last_reviewed: 2026-03-01
tags:
  - operations
  - runtime
related:
  - docs/reference/index.md
  - docs/architecture/index.md
---

# Operations

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@50be979f`
- Reason to exist: provide the canonical operator entrypoint across run, deploy, observe, and incident workflows.

## Operator Entry

1. [Operations overview](overview.md)
2. [Run locally](run-locally.md)
3. [Deploy](deploy.md)
4. [Minimal production overrides](minimal-production-overrides.md)
5. [Install verification checklist](install-verification-checklist.md)
6. [Observability setup](observability-setup.md)
7. [Incident response](incident-response.md)
8. [Release](release/index.md)
9. [Security posture](security-posture.md)

## What This Page Is Not

This page is not a command reference and not an architecture deep dive.
Operational policies are enforced by contracts such as `OPS-ROOT-023` and `OPS-ROOT-017`.
The docs surface stays in `docs/operations/**`; contract sources stay in `docs/_internal/contracts/**`.

## Verify Success

Operator workflows are successful when each linked page reaches a concrete verification outcome.

## Next steps

Use [Reference](../reference/index.md) for exact flags and schemas, and [Runbooks](runbooks/index.md) during incidents.
Also review [Glossary](../glossary.md) for canonical terms.
For product intent and boundaries, read [What is Bijux Atlas](../product/what-is-bijux-atlas.md).

## Document Taxonomy

- Audience: `operator`
- Type: `guide`
- Stability: `stable`
- Owner: `bijux-atlas-operations`
