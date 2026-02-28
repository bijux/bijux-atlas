# Runtime Platform

Owner: `bijux-atlas-operations`  
Audience: `operator`  
Type: `runbook`  
Reason to exist: unify container and runtime configuration operations in one canonical page.

## Container Contract

- Use pinned image tags or digests.
- Validate runtime image contract before deployment.
- Do not use mutable `latest` tags.

## Runtime Config Contract

- Runtime config changes require explicit rollout.
- Unknown keys are rejected by strict config checks.
- Config version stamps are mandatory for live provenance.

## Verification

```bash
make docker-gate
make k8s/restart
```
