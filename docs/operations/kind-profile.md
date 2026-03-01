# Kind profile quickstart

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: provide the fastest local Kubernetes path for validating install, readiness, and smoke checks.

## Profiles

- `dev`: small local cluster for smoke checks
- `perf`: larger local cluster for capacity-style rehearsals
- `small`: constrained profile for lower-resource laptops

## Source of truth

- `ops/stack/profiles.json`
- `ops/stack/kind/cluster-dev.yaml`
- `ops/stack/kind/cluster-perf.yaml`
- `ops/stack/kind/cluster-small.yaml`

## Quickstart

```bash
make ops-prereqs
make kind-up
make ops-deploy
make ops-k8s-smoke
```

## Verify success

```bash
make ops-readiness-scorecard
make ops-e2e-smoke
```

Expected outputs:

- kind cluster starts successfully
- deploy reaches readiness
- smoke checks confirm the route is serving

## Rollback

```bash
make stack-down
make kind-down
```

## Next

- [Deploy to kind (10 minutes)](deploy-kind.md)
- [Kubernetes E2E guarantees](e2e/k8s-tests.md)
- [Load testing](load/index.md)
