# Data Protection Best Practices

- Owner: `bijux-atlas-security`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: summarize durable operational patterns.

## Practices

- fail closed on TLS, integrity, and authorization checks
- treat tamper signals as incident-level events
- keep cryptographic material short-lived and rotated
- review retention settings for audit and integrity evidence
- run data-protection diagnostics before release promotion
