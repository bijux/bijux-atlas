# Incident Response

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: provide one canonical response flow for production incidents.

## Response Flow

1. Detect and classify incident severity.
2. Stabilize user-facing impact.
3. Diagnose cause using logs, traces, and metrics.
4. Apply mitigation or rollback.
5. Escalate when the runbook ceiling is reached.
6. Record timeline and follow-up actions.

## Policy Violation Triage

- Confirm violation source and blast radius.
- Contain impact before broad policy changes.
- Apply minimal safe mitigation and record evidence.
- Escalate if mitigation requires contract bypass.

## Canonical Details

- [Incident Playbook](runbooks/incident-playbook.md)
- [Observability Overview](observability/overview.md)
