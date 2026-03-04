# Kubernetes Operations

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide the single Kubernetes entrypoint for Atlas operators.

## Purpose

Run Atlas reliably on Kubernetes with one path for deploy, health checks, and failure response.

## What you will find here

- Cluster deployment path in [Deploy](../deploy.md)
- Release controls in [Release Workflow](../release-workflow.md)
- K8s validation in [E2E Kubernetes Tests](../e2e/k8s-tests.md)
- Install profile guardrails in [Profile Invariants](profile-invariants.md)
- Chart-level invariants in [Chart Contracts](chart-contracts.md)
- Network policy modes in [NetworkPolicy](../networkpolicy.md)
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
