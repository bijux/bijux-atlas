# Load Failure Triage

Owner: `bijux-atlas-operations`  
Type: `runbook`  
Reason to exist: provide deterministic incident response steps for Load Failure Triage events.

## Symptoms

- Key user-visible and operational signals indicating this condition.

## Diagnosis

1. Confirm health and readiness state.
2. Inspect logs, traces, and metrics for the failing component.
3. Verify recent deployment or config changes.

## Mitigation

1. Apply the safest immediate stabilization action.
2. Reduce blast radius while preserving critical read paths.

## Rollback

- Revert the latest risky deployment or config pointer if mitigation is insufficient.

## Escalation

- Escalate to platform owner when mitigation and rollback do not restore service.

## What Changed

- 2026-02-28: normalized runbook structure and canonical response flow.
