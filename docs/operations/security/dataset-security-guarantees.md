# Dataset Security Guarantees

- Owner: `bijux-atlas-security`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: state runtime guarantees for dataset security.

## Guarantees

- authenticated and authorized access for protected endpoints
- checksum validation before serving cached dataset artifacts
- corruption re-verification and quarantine on integrity failures
- auditability for denied authorization and security-sensitive events
