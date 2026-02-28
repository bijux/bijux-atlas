# Autoscaling

Owner: `bijux-atlas-operations`  
Type: `runbook`  
Reason to exist: define autoscaling expectations and safe rollout behavior.

## Rules

- Scale policy must preserve readiness guarantees.
- Scale events must not bypass health gates.
