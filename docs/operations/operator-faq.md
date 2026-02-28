# Operator FAQ

Owner: `bijux-atlas-operations`  
Type: `runbook`  
Reason to exist: provide concise answers for recurring on-call operational questions.

## Why does readiness fail while health passes?

Readiness requires downstream dataset and catalog availability; health only indicates process liveness.

## When should rollback be preferred over mitigation?

Use rollback when mitigation does not quickly restore stable user-facing behavior.

## How are policy-violation incidents handled?

Triage in the incident response flow, contain impact, and escalate before policy bypass.
