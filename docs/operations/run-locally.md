# Run Locally

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@50be979f`
- Reason to exist: provide one canonical local workflow from prerequisites through cleanup.

## Prerequisites

- Container runtime is installed and healthy.
- Required local tooling is available.
- Fixture dataset source is available.

## Start

```bash
make ops-doctor
make stack-up
```

## Verify Success

```bash
make ops-e2e-smoke
make ops-observability-verify
```

Expected outcome: local stack responds and smoke checks pass.

## Stop

```bash
make stack-down
```

## Cleanup

```bash
make ops-clean
```

## Rollback

No rollback is required for local-only workflows; stop and clean reset the environment.

## Next

- Deployment workflow: [Deploy](deploy.md)
- Incident process: [Incident Response](incident-response.md)
