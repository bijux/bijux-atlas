# Security Deployment Examples

- Owner: `bijux-atlas-security`
- Type: `example`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide deployment examples with security controls.

## Ingress And Runtime

- ingress enforces HTTPS
- ingress forwards only approved identity headers
- runtime enforces authorization policy and role contracts
- runtime rejects integrity failures during dataset ingest and reverify
