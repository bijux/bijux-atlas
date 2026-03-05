# Incident Response Playbook

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Trigger

Activate when SLO burn or error classification dashboards exceed alert budgets.

## Response

1. Confirm incident scope from `atlas-slo-compliance` and `atlas-error-classification` dashboards.
2. Identify affected domain dashboard and capture evidence.
3. Execute mitigation from service runbooks.
4. Validate recovery in latency and runtime dashboards.
5. Record incident timeline and follow-up actions.
