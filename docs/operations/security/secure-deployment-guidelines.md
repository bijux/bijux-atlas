# Secure Deployment Guidelines

- Owner: `bijux-atlas-security`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: provide baseline secure deployment controls.

## Baseline Controls

- enforce HTTPS transport in front of data-plane routes
- deploy behind trusted authentication boundary for OIDC or mTLS modes
- configure role and permission contracts before exposing endpoints
- enable audit logging and bounded retention
- monitor integrity/tamper counters and alerts

## Data Protection Controls

- use TLS certificate validation for ingress and internal hops
- enable artifact integrity verification and corruption quarantine
- follow sensitive data handling and redaction rules
