---
title: Docs Consolidation
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

# Docs Consolidation

## Canonical Pages

- Domain indexes:
  - `docs/index.md`
  - `docs/product/index.md`
  - `docs/operations/index.md`
  - `docs/development/index.md`
  - `docs/control-plane/index.md`
  - `docs/reference/index.md`
- Reading progression: `docs/what-to-read-next.md`
- Compatibility promise: `docs/product/compatibility-promise.md`
- Docs style contract: `docs/_internal/governance/docs-style.md`

## Consolidation Map

| Source | Canonical target | Reason |
| --- | --- | --- |
| `docs/root/index.md` | `docs/index.md` | one published root index |
| `docs/dev/index.md` | `docs/development/index.md` | one development domain index |
| `docs/security/index.md` | `docs/operations/security/index.md` | security docs live under operations |
| `docs/science/index.md` | `docs/product/index.md` | product domain supersedes historical science index |
| `docs/product/reading-tracks.md` | `docs/what-to-read-next.md` | one reading progression page |
| `docs/root/compatibility-policy.md` | `docs/product/compatibility-promise.md` | one compatibility promise narrative |
| `docs/root/reproducibility-guarantees.md` | `docs/product/compatibility-promise.md` | merged with compatibility promise scope |
| `docs/root/stability-guarantees.md` | `docs/product/compatibility-promise.md` | merged with compatibility promise scope |
| `docs/contracts/compatibility.md` | `docs/reference/contracts/compatibility.md` | one contracts compatibility reference |

## Rules

- Consolidation keeps one canonical page per concept and uses redirects for compatibility.
- Each major domain keeps one canonical `index.md`.
- New duplicate narrative pages are not allowed when an existing canonical page exists.
- Canonical merge decisions are tracked in `docs/_internal/governance/docs-merge-plan.md`.
