# Runbooks to alerts mapping

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: keep the alert-to-runbook routing table in one canonical, linkable page.

| Alert condition | Runbook URL |
| --- | --- |
| Store backend error spike | [Store backend error spike](runbooks/slo-store-backend-error-spike.md) |
| Store outage | [Store outage](runbooks/store-outage.md) |
| Dataset corruption signal | [Dataset corruption](runbooks/dataset-corruption.md) |
| Federation sync failure | [Registry federation](runbooks/registry-federation.md) |
| Sustained traffic overload | [Traffic spike](runbooks/traffic-spike.md) |
| Load gate failure | [Load failure triage](runbooks/load-failure-triage.md) |
| Rollback required by deploy guard | [Rollback playbook](runbooks/rollback-playbook.md) |
| Unknown multi-surface incident | [Incident playbook](runbooks/incident-playbook.md) |

## Verify success

```bash
make ops-alerts-validate
```

## Next

- [Alerts](observability/alerts.md)
- [Runbooks](runbooks/index.md)
- [Incident response](incident-response.md)
