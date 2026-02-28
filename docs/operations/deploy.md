# Deploy

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: define the canonical Atlas deployment workflow.

## Deployment Flow

1. Validate release inputs and compatibility requirements.
2. Apply deployment manifests and configuration.
3. Verify readiness, health, and error budgets.
4. Confirm post-deploy smoke checks.

## Canonical Details

- [Kubernetes Operations](kubernetes.md)
- [Kubernetes Index](k8s/INDEX.md)
- [Release Workflows](release-workflows.md)
- [Production Readiness Checklist](production-readiness-checklist.md)
