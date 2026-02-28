# Deploy to kind (10 minutes)

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@fbf7e658`
- Reason to exist: provide a single verified deployment path for local Kubernetes testing on kind.

## Prerequisites

- `kind` `v0.23+`
- `kubectl` `v1.30+`
- `helm` `v3.15+`
- Docker runtime running

## Deploy

```bash
make kind-up
make ops-deploy
```

## What gets installed

- Atlas runtime workloads
- service and networking resources required for local cluster access
- readiness and observability wiring used by operator checks

## Verify success

```bash
make ops-readiness-scorecard
make ops-e2e-smoke
```

Expected outputs:

- all workload pods report `Running`
- readiness checks pass
- smoke checks return exit code `0`

## How to interpret success

- Running pods plus readiness checks indicate chart values and runtime config are compatible.
- Passing smoke checks indicate API and query paths are reachable in-cluster.
- A successful run means the cluster is valid for release rehearsal or regression testing.

## Where artifacts and logs live

- Control-plane artifacts: `artifacts/`
- Kubernetes events/logs: `kubectl` logs and describe output for deployed namespace
- Generated diagnostics: `docs/_generated/` for contributor-only quality traces

## Reset and cleanup

```bash
make stack-down
make kind-down
make ops-clean
```

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `ImagePullBackOff` | image not available in kind nodes | build/push expected image and redeploy |
| readiness probe fails | config or dependency mismatch | run `make ops-readiness-scorecard`, fix config, redeploy |
| e2e smoke fails | service endpoint not reachable | check service and ingress, then rerun smoke |

## Next steps

- Production-like deployment: [Deploy to Kubernetes (prod minimal)](deploy-kubernetes-minimal.md)
- Incident handling: [Incident response](incident-response.md)
- architecture context: [Architecture](../architecture/index.md)
- contributor workflows: [Development](../development/index.md)
