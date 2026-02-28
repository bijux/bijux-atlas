# Start here

- Owner: `docs-governance`
- Type: `guide`
- Audience: `user`
- Stability: `stable`
- Last verified against: `main@699e8b0e`
- Reason to exist: provide the only onboarding funnel for Atlas.

This is the only onboarding root in `docs/`.

## 5-minute mental model

Atlas validates dataset inputs, builds immutable artifacts, and serves stable API queries through explicit operational controls.

## Quickstart

```bash
bijux dev atlas demo quickstart --format json
```

## Run locally

- [ ] Follow [Run locally (5 minutes)](operations/run-locally.md)
- [ ] Start stack and run smoke checks
- [ ] Verify success and read outputs

## Deploy

- [ ] Run [Deploy to kind (10 minutes)](operations/deploy-kind.md)
- [ ] Run [Deploy to Kubernetes (prod minimal)](operations/deploy-kubernetes-minimal.md)
- [ ] Verify observability and rollback controls

## Extend

- [ ] Read [Development](development/index.md)
- [ ] Review [Control plane](development/control-plane.md)
- [ ] Follow [Contributing](development/contributing.md)

## Next steps

- API integration: [API](api/index.md)
- Runtime model: [Architecture dataflow](architecture/dataflow.md)
- Operator workflows: [Operations](operations/index.md)
- Contributor workflows: [Development](development/index.md)
- Guided tracks: [Reading tracks](product/reading-tracks.md)
- Onboarding checklist: [Contributor onboarding checklist (30 minutes)](development/contributor-onboarding-checklist.md)
- Terms: [Glossary](glossary.md)
