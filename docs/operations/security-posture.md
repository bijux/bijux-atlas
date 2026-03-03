# Security Posture

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define operator-focused RBAC, secrets, and network policy expectations.

## Why you are reading this

Use this page to validate the security baseline before and after deployment changes.

## RBAC

- Service accounts must use least privilege roles.
- Cluster-scoped permissions require explicit review evidence.
- Contract reference: `OPS-K8S-005`.

## Secrets

- Secrets must be provided by approved secret stores.
- No plaintext credentials in values files or docs examples.
- Contract reference: `OPS-ROOT-023`.

## Network policies

- Ingress and egress are deny-by-default except approved service flows.
- Observability egress endpoints must be explicitly listed.
- Minimal privileges apply to service accounts, secrets access, and chart-provided RBAC bindings.
- Contract reference: `OPS-K8S-004`.

## Auth boundary

- Default auth stance: `internal`.
- Built-in auth modes: `disabled`, `api-key`, `hmac`.
- If built-in auth is disabled, Atlas must remain behind an ingress auth proxy or equivalent boundary.
- Primary references: `docs/architecture/security/auth-model.md` and
  `docs/operations/security/deploy-behind-auth-proxy.md`.

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
