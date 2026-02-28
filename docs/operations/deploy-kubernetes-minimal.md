# Deploy to Kubernetes (prod minimal)

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@fbf7e658`
- Reason to exist: define the minimal production deployment recipe with explicit verification.

## Prerequisites

- `kubectl` `v1.30+`
- `helm` `v3.15+`
- target namespace exists
- release image digest approved
- `values-prod-minimal.yaml` prepared

## Deploy

```bash
make ops-deploy
```

Apply minimal production overrides:

- image digest pin
- replica count
- cpu and memory requests/limits
- storage class and persistence size
- telemetry endpoints

## What gets installed

- Atlas runtime workloads for production baseline
- persistent storage bindings required by serving components
- service, ingress, and telemetry wiring for operational visibility

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outputs:

- all required pods ready
- probes healthy
- metrics and logs visible

## How to interpret success

- Ready pods and healthy probes indicate deploy-time config is valid.
- Visible telemetry confirms observability endpoints and collectors are correctly configured.
- A passing verification run indicates rollout can proceed to release workflow controls.

## Reset and cleanup

```bash
make stack-down
```

Use rollback workflow for failed production changes.

## Where artifacts and logs live

- Control-plane artifacts: `artifacts/`
- Cluster diagnostics: `kubectl` events, logs, and rollout status
- Release evidence: operator reports linked from release workflow and runbooks

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| pods pending | insufficient cluster resources | adjust requests/limits and node capacity |
| crash loop | invalid runtime config | compare chart values to [Reference configs](../reference/configs.md) |
| observability missing | telemetry endpoint misconfigured | verify endpoint values and redeploy |

## Next steps

- Release controls: [Release workflow](release-workflow.md)
- Rollback controls: [Rollback procedure](release/rollback-procedure.md)
- architecture context: [Architecture](../architecture/index.md)
- contributor workflows: [Development](../development/index.md)
