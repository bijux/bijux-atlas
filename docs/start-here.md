---
title: Start here
audience: user
type: how-to
stability: stable
owner: docs-governance
last_reviewed: 2026-03-01
tags:
  - onboarding
  - quickstart
related:
  - docs/index.md
  - docs/what-to-read-next.md
verification: true
prerequisites:
  - make
  - cargo
---

# Start here

- Owner: `docs-governance`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide the only onboarding funnel for Atlas.

This is the only onboarding root in `docs/`.

## 5-minute mental model

Atlas validates dataset inputs, builds immutable artifacts, and serves stable API queries through explicit operational controls.

## Primary proof path

Start with [Real Data Runs](tutorials/real-data/index.md). This is the fastest path to verify end-to-end ingest, query, and evidence behavior on real datasets.

## Quickstart

```bash
bijux dev atlas demo quickstart --format json
```

## Run locally

- [ ] Follow [Run locally (5 minutes)](operations/run-locally.md)
- [ ] Start stack and run smoke checks
- [ ] Verify success and read outputs
- [ ] Run tutorial workflow via `bijux-dev-atlas tutorials run workflow`
- [ ] Verify SDK surface via `bijux-dev-atlas clients verify --client atlas-client`

## Deploy

- [ ] Run [Deploy to kind (10 minutes)](operations/deploy-kind.md)
- [ ] Run [Deploy to Kubernetes (prod minimal)](operations/deploy-kubernetes-minimal.md)
- [ ] Verify observability and rollback controls

## Extend

- [ ] Read [Development](development/index.md)
- [ ] Review [Control-plane](control-plane/index.md)
- [ ] Follow [Contributing](development/contributing.md)

## Next steps

- [Real data runs](tutorials/real-data/index.md)
- [Reviewer quick path](tutorials/real-data/reviewer-quick-path.md)
- [Product Index](product/index.md)
- [Architecture Index](architecture/index.md)
- [Operations Index](operations/index.md)
- [Reference Index](reference/index.md)
- [What we built](what-we-built.md)
