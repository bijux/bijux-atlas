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
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Last changed: `2026-03-03`
- Reason to exist: provide the canonical operator entrypoint across run, deploy, observe, and incident workflows.

## Purpose

Give operators one stable entrypoint for deploy, observe, rollback, and incident handling.

## Entry Points

1. [Operations overview](overview.md)
2. [Operator quickstart](operator-quickstart.md)
3. [Run locally](run-locally.md)
4. [Deploy](deploy.md)
5. [Minimal production overrides](minimal-production-overrides.md)
6. [Install verification checklist](install-verification-checklist.md)
7. [Observability setup](observability-setup.md)
8. [Validation entrypoints](validation-entrypoints.md)
9. [Kind simulation](kind-simulation.md)
10. [Upgrade validation](upgrade.md)
11. [Chart semver policy](chart-semver-policy.md)
12. [Profile upgrade policy](profile-upgrade-policy.md)
13. [Rollback limitations](rollback-limitations.md)
14. [Incident response](incident-response.md)
15. [Release](release/index.md)
16. [Release evidence](release-evidence.md)
17. [Release candidate checklist](release-candidate-checklist.md)
18. [Evidence viewer](evidence-viewer.md)
19. [Security posture](security-posture.md)

## What This Page Is Not

This page is not a command reference and not an architecture deep dive.
Operational policies are enforced by contracts such as `OPS-ROOT-023` and `OPS-ROOT-017`.
The docs surface stays in `docs/operations/**`; contract sources stay in `docs/_internal/contracts/**`.

## Verify Success

Operator workflows are successful when each linked page reaches a concrete verification outcome.

## Next steps

Use [Reference](../reference/index.md) for exact flags and schemas, and [Runbooks](runbooks/index.md) during incidents.
Also review the glossary for canonical terms.
For product intent and boundaries, read [What is Bijux Atlas](../product/what-is-bijux-atlas.md).

## Document Taxonomy

- Audience: `operator`
- Type: `guide`
- Stability: `stable`
- Owner: `bijux-atlas-operations`
