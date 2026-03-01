# Run locally (5 minutes)

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@fbf7e658`
- Reason to exist: provide one canonical 5-minute local workflow with explicit verification and recovery guidance.

## Prerequisites

- Docker or compatible runtime healthy
- `make` available
- `kubectl` `v1.30+`
- `helm` `v3.15+`

## Run

```bash
make ops-doctor
make stack-up
```

## Verify success

```bash
make ops-e2e-smoke
make ops-observability-verify
```

Expected outputs:

- stack services are healthy
- smoke checks return exit code `0`
- observability checks confirm baseline metrics and logs

## How to interpret success

- Health and smoke checks passing means runtime dependencies are wired correctly.
- Observability checks passing means metrics/log pipelines are reachable.
- A clean `0` exit code across checks means local operator workflow is ready for deploy-path progression.

## Stop

```bash
make stack-down
```

## Reset and cleanup

```bash
make ops-clean
```

## Where artifacts and logs live

- Control-plane artifacts: `artifacts/`
- Runtime logs: container runtime logs for stack services
- Verification output: terminal output from `make ops-e2e-smoke` and `make ops-observability-verify`

## Common failures and fixes

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `make stack-up` fails | local runtime unavailable | start docker runtime, rerun `make ops-doctor` |
| smoke checks fail | stale local state | run cleanup commands and restart stack |
| observability verify fails | telemetry service not ready | wait for readiness and rerun `make ops-observability-verify` |

## Next steps

- kind deployment: [Deploy to kind (10 minutes)](deploy-kind.md)
- production deployment: [Deploy to Kubernetes (prod minimal)](deploy-kubernetes-minimal.md)
- incident process: [Incident response](incident-response.md)
- architecture context: [Architecture](../architecture/index.md)
- contributor workflows: [Development](../development/index.md)
