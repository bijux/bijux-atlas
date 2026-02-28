# Kubernetes Operations

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: provide the single Kubernetes entrypoint for Atlas operators.

## Purpose

Run Atlas reliably on Kubernetes with one path for deploy, health checks, and failure response.

## What you will find here

- Cluster deployment path in [Deploy](../deploy.md)
- Release controls in [Release Workflow](../release-workflow.md)
- K8s validation in [E2E Kubernetes Tests](../e2e/k8s-tests.md)
- Incident actions in [Incident Response](../incident-response.md)

## Verify success

```bash
make k8s-validate
make ops-k8s-tests
```

Expected result: chart linting and Kubernetes test suites pass.

## Rollback

If deployment checks fail, follow [Rollback Playbook](../runbooks/rollback-playbook.md).

## Next

- [Observability](../observability/index.md)
- [Runbooks](../runbooks/index.md)
