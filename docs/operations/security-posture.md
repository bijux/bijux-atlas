# Security Posture

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@8641e5b0`
- Reason to exist: define operator-focused RBAC, secrets, and network policy expectations.

## Why you are reading this

Use this page to validate the security baseline before and after deployment changes.

## RBAC

- Service accounts must use least privilege roles.
- Cluster-scoped permissions require explicit review evidence.

## Secrets

- Secrets must be provided by approved secret stores.
- No plaintext credentials in values files or docs examples.

## Network policies

- Ingress and egress are deny-by-default except approved service flows.
- Observability egress endpoints must be explicitly listed.
- Minimal privileges apply to service accounts, secrets access, and chart-provided RBAC bindings.

## Verify success

```bash
make ops-k8s-tests
make ops-observability-verify
make ops-tools-check
```

Expected result: policy checks pass and no forbidden network paths are reported.

## Rollback

If a security baseline change widens privileges or breaks cluster safety, revert the change and rerun the policy checks before redeploy.

## Next

- [Security](security/index.md)
- [Incident Response](incident-response.md)
