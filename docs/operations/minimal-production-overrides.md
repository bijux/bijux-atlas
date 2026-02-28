# Minimal production overrides

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2026-03-01`
- Reason to exist: define the smallest Helm values override set needed for a production-safe Atlas deployment.

## Required overrides

- image digest pin
- namespace and release name
- replica count
- cpu and memory requests and limits
- persistent volume size and storage class
- ingress and service exposure
- metrics, logs, and trace endpoints

## Verify success

```bash
make ops-prereqs
make ops-values-validate
make ops-observability-verify
```

Expected outputs:

- install plan resolves cleanly
- telemetry targets appear in the generated plan

## Rollback

Use [Rollback procedure](release/rollback-procedure.md) if the applied overrides regress readiness or telemetry.

## Next steps

- [Deploy](deploy.md)
- [Values mapping to config keys](values-mapping-to-config-keys.md)
- [Install verification checklist](install-verification-checklist.md)
