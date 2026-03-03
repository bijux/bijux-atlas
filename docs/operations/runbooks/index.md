# Runbooks

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Last changed: `2026-03-03`
- Reason to exist: provide the essential incident runbook catalog and alert mapping.

## Prereqs

- Access to alerting, cluster context, and operator evidence commands.

## Install

- Select the runbook that matches the active failure mode and begin the documented response.

## Runbook catalog

- [Incident Playbook](incident-playbook.md)
- [Store Backend Error Spike](slo-store-backend-error-spike.md)
- [Store Outage](store-outage.md)
- [Dataset Corruption](dataset-corruption.md)
- [Registry Federation](registry-federation.md)
- [Traffic Spike](traffic-spike.md)
- [Load Failure Triage](load-failure-triage.md)
- [Rollback Playbook](rollback-playbook.md)

## Runbook-to-alert mapping

See [Runbooks to alerts mapping](../runbooks-to-alerts-mapping.md) for the explicit routing table used by alerts.

## Alert mapping

See [Observability Alerts](../observability/alerts.md) for alert-to-runbook routing.

## Verify

The selected runbook produces a clear mitigation path and a concrete success signal.

## Symptoms

This index aggregates recurring incident symptoms and routes to a specific runbook.

## Metrics

Primary SLO and alert metrics are defined in the linked runbooks and observability pages.

## Commands

```bash
make ops-readiness-scorecard
```

## Expected outputs

Readiness scorecard output should summarize lane health and artifact evidence.

## Mitigations

Select the runbook that matches the active alert and follow its mitigation flow.

## Rollback

Use [Rollback Playbook](rollback-playbook.md) when mitigations require controlled rollback.

## Postmortem checklist

- Capture timeline and impacted surfaces.
- Link incident to the executed runbook.
- Record follow-up actions in release planning.

## Next

- [Incident Response](../incident-response.md)
- [Observability](../observability/index.md)
