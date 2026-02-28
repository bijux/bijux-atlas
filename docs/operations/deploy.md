# Deploy

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: define canonical deployment flow for production and staging clusters.

## Deployment Flow

1. Validate release and config inputs.
2. Apply chart values and manifests.
3. Verify readiness and health gates.
4. Run post-deploy smoke checks.
5. Promote release and record evidence.

## Deep Dives

- [Release Workflow](release-workflow.md)
- [Incident Response](incident-response.md)
- [Runbooks](runbooks/incident-playbook.md)

## Container Governance

Container images and Docker build policy are defined under:

- `docker/README.md`
- `docker/CONTRACT.md`
