---
title: Docs Merge Plan
audience: internal
type: policy
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - docs
  - consolidation
  - ssot
---

# Docs Merge Plan

This table records canonical consolidation decisions and the redirect target for each absorbed page.

| Source page | Canonical target | Status |
| --- | --- | --- |
| `docs/root/repository-structure.md` | `docs/development/repo-layout.md` | completed |
| `docs/root/ci-workflow-explanation.md` | `docs/development/ci-overview.md` | completed |
| `docs/dev/check-failures.md` | `docs/development/debugging-locally.md` | completed |
| `docs/root/terminology-rules.md` | `docs/_internal/governance/docs-style.md` | completed |
| `docs/governance/docs-ownership.md` | `docs/_internal/governance/docs-operating-model.md` | completed |
| `docs/product/reading-tracks.md` | `docs/what-to-read-next.md` | completed |
| `docs/root/compatibility-policy.md` | `docs/product/compatibility-promise.md` | completed |
| `docs/control-plane/add-a-check-in-30-minutes.md` + `docs/control-plane/add-a-contract-registry-in-30-minutes.md` | `docs/control-plane/extend-control-plane.md` | completed |
| `docs/operations/reference/commands.md` | `docs/reference/commands.md` | completed |
| `docs/reference/errors.md` | `docs/reference/errors-and-exit-codes.md` | completed |
| API lifecycle narrative consolidation | `docs/api/lifecycle.md` | completed |
| repeated deployment prerequisites sections | `docs/operations/prerequisites.md` | completed |
| repeated FAQ entrypoints | `docs/product/faq.md` | completed |

## Guardrails

- Keep one canonical page per concept.
- Preserve reader URLs with redirects while inbound links are migrated.
- Reject new duplicate narrative pages during review.
