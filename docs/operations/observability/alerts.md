# Alerts

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@c59da0bf`
- Reason to exist: map every production alert to an actionable runbook URL.

## Alert routing

| Alert condition | Severity | Runbook |
| --- | --- | --- |
| Store backend error spike | page | [Store Backend Error Spike](../runbooks/slo-store-backend-error-spike.md) |
| Store outage | page | [Store Outage](../runbooks/store-outage.md) |
| Dataset corruption signal | page | [Dataset Corruption](../runbooks/dataset-corruption.md) |
| Federation sync failure | page | [Registry Federation](../runbooks/registry-federation.md) |
| Sustained traffic overload | page | [Traffic Spike](../runbooks/traffic-spike.md) |
| Load gate failure | ticket | [Load Failure Triage](../runbooks/load-failure-triage.md) |
| Rollback required by deploy guard | page | [Rollback Playbook](../runbooks/rollback-playbook.md) |
| Unknown multi-surface incident | page | [Incident Playbook](../runbooks/incident-playbook.md) |

## Alert source of truth

- `ops/observe/alerts/atlas-alert-rules.yaml`
- `ops/observe/alerts/slo-burn-rules.yaml`

## Verify success

```bash
make ops-observability-verify
```

Expected result: each active alert resolves to one runbook URL.

## Next

- [Incident Response](../incident-response.md)
- [Runbooks](../runbooks/index.md)
