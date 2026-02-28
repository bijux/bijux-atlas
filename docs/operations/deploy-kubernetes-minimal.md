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

## Verify success

```bash
make ops-readiness-scorecard
make ops-observability-verify
```

Expected outputs:

- all required pods ready
- probes healthy
- metrics and logs visible

## Reset and cleanup

```bash
make stack-down
```

Use rollback workflow for failed production changes.

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| pods pending | insufficient cluster resources | adjust requests/limits and node capacity |
| crash loop | invalid runtime config | compare chart values to [Reference configs](../reference/configs.md) |
| observability missing | telemetry endpoint misconfigured | verify endpoint values and redeploy |

## Next steps

- Release controls: [Release workflow](release-workflow.md)
- Rollback controls: [Rollback procedure](release/rollback-procedure.md)
