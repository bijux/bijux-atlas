---
title: Home
audience: user
type: concept
stability: stable
owner: docs-governance
last_reviewed: 2026-03-01
tags:
  - onboarding
  - navigation
related:
  - docs/start-here.md
  - docs/site-map.md
---

# Home

- Owner: `docs-governance`

`docs/INDEX.md` is the canonical top-level navigation authority for reader-facing docs.

Atlas documentation is organized by system understanding first, then execution detail.

## What Exists

| Surface | Location | Purpose |
| --- | --- | --- |
| Crates | `crates/` | runtime, ingest, query, API, and tooling |
| Ops | `ops/` | deploy profiles, schemas, and release evidence |
| Docs | `docs/` | canonical reader and operator guides |
| Docker | `docker/` | image build and runtime definitions |

## What Is Proven

- Ten real-data runs: [Real Data Runs](tutorials/real-data/index.md)
- Combined run report: [Real data runs report](reference/reports/real-data-runs.md)
- Contract coverage: [_internal generated coverage](_internal/generated/docs-contract-coverage.json)

## Evidence-first Evaluation Path

1. `bijux-dev-atlas tutorials list-runs --format json`
2. `bijux-dev-atlas tutorials plan-run --run-id rdr-001-genes-baseline --format json`
3. `bijux-dev-atlas tutorials docs-report --format json`
4. `bijux-dev-atlas docs generate real-data-pages --allow-write --format json`

## Primary Reader Paths

- New to Atlas: [Architecture Summary](architecture/summary.md)
- System understanding: [Architecture Overview](architecture/overview.md)
- Runtime operation: [Deploy Overview](operations/deploy.md)
- API consumer path: [API v1 Surface](api/v1-surface.md)
- Contributor path: [Contributor Onboarding Rubric](development/contributor-onboarding-rubric.md)
- Reviewer path: [For reviewers](product/for-reviewers.md)
- Operator path: [For operators](product/for-operators.md)

## Docs Spine

- Start: [Start Here](start-here.md)
- Product: [What Is Bijux Atlas](product/what-is-bijux-atlas.md)
- Product Index: [Product Index](product/index.md)
- Architecture: [Architecture Index](architecture/index.md)
- API: [API Surface Index](api/index.md)
- Ops: [Operations Index](operations/index.md)
- Dev: [Development Index](development/index.md)
- Control-plane: [Control-plane Index](control-plane/index.md)
- Reference: [Reference Index](reference/index.md)
- Governance: [Governance Index](governance/index.md)

## Next steps

- Platform summary: [What We Built](what-we-built.md)
- Run locally: [Run locally](operations/run-locally.md)
- Deploy to kind: [Deploy to kind](operations/deploy-kind.md)
- Deploy to Kubernetes: [Deploy to Kubernetes](operations/deploy-kubernetes-minimal.md)
- Stability values: [Stability legend](reference/stability-legend.md)
- Audience values: [Audience legend](reference/audience-legend.md)
- [Site map](site-map.md)
- [What to read next](what-to-read-next.md)
- [Glossary](glossary.md)
