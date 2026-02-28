# Deploy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@fbf7e658`
- Reason to exist: define the canonical Helm-based deployment path and route operators to the right verification and override guides.

## Prerequisites

- Release artifact is published and approved.
- Cluster access and namespace policies are configured.
- Required chart values are prepared.
- `helm`, `kubectl`, and required runtime credentials are available.

## Golden path

1. Prepare overrides with [Minimal production overrides](minimal-production-overrides.md).
2. Render the plan before apply.
3. Apply the install or upgrade through the canonical wrapper.
4. Run the install verification checklist.

```bash
make ops-prereqs
make ops-deploy
make ops-readiness-scorecard
```

## What gets installed

- Atlas workloads and services
- readiness and observability wiring
- chart-rendered runtime configuration for the selected profile

## Values mapping

Chart values map to runtime config keys documented in [Reference configs](../reference/configs.md) and [Values mapping to config keys](values-mapping-to-config-keys.md).

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
make ops-e2e-smoke
```

Expected outputs:

- workloads ready
- probes healthy
- observability checks pass
- smoke or e2e validation succeeds

## How to interpret success

- healthy pod status means the chart and cluster resources are compatible
- observability success means operators have enough signal to support the install
- smoke or e2e success means the system is serving through the deployed path, not just starting

## Rollback

```bash
make ops-release-rollback
```

Use [Rollback procedure](release/rollback-procedure.md) for production rollbacks.

## Reset and cleanup

```bash
make ops-clean
```

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| rollout timeout | insufficient resources or bad image | inspect events, adjust resources, rerun `make ops-deploy` |
| readiness failures | config mismatch | validate values against reference configs and run `make ops-readiness-scorecard` again |
| no metrics after deploy | observability endpoint misconfigured | update telemetry values and rerun `make ops-observability-verify` |

## Next steps

- [Deploy to kind (10 minutes)](deploy-kind.md)
- [Deploy to Kubernetes (prod minimal)](deploy-kubernetes-minimal.md)
- [Install verification checklist](install-verification-checklist.md)
- [Incident response](incident-response.md)
