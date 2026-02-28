# Deploy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@fbf7e658`
- Reason to exist: route operators to the two canonical deployment golden paths with shared verification rules.

## Prerequisites

- Release artifact is published and approved.
- Cluster access and namespace policies are configured.
- Required chart values are prepared.

## Choose deployment path

- Local Kubernetes test path: [Deploy to kind (10 minutes)](deploy-kind.md)
- Production baseline path: [Deploy to Kubernetes (prod minimal)](deploy-kubernetes-minimal.md)

## Values Mapping

Chart values map to runtime config keys documented in [Reference Configs](../reference/configs.md).

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outputs:

- workloads ready
- probes healthy
- observability checks pass

## Rollback

```bash
make stack-down
```

Use [Rollback procedure](release/rollback-procedure.md) for production rollbacks.

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| rollout timeout | insufficient resources or bad image | inspect events, adjust resources, redeploy |
| readiness failures | config mismatch | validate values against reference configs |
| no metrics after deploy | observability endpoint misconfigured | update telemetry values and redeploy |

## Next steps

- [Release Workflow](release-workflow.md)
- [Incident Response](incident-response.md)
- [K8s](k8s/index.md)
