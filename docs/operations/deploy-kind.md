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

## Verify success

```bash
make ops-readiness-scorecard
make ops-e2e-smoke
```

Expected outputs:

- all workload pods report `Running`
- readiness checks pass
- smoke checks return exit code `0`

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
