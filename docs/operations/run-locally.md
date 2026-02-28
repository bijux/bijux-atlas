# Run Locally

Owner: `bijux-atlas-operations`  
Audience: `operator`, `contributor`  
Type: `runbook`  
Reason to exist: provide one canonical local workflow from prerequisites through cleanup.

## Prerequisites

- Container runtime installed and healthy.
- Required local tooling available.
- Fixture dataset source available.

## Workflow

1. Start local stack.
2. Ingest fixture dataset.
3. Run API smoke checks.
4. Verify health and metrics.
5. Stop stack and clean temporary artifacts.

## Verification

```bash
make doctor
make ops-doctor
make stack-up
```

## Cleanup

```bash
make ops-clean
```

## Next Step

- Deployment workflow: [Deploy](deploy.md)
- Incident process: [Incident Response](incident-response.md)
