# Data Retention Policy

- Owner: `bijux-atlas-security`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Reason to exist: define retention windows for security and integrity records.

## Retention Windows

- audit records: 30 days
- security event records: 90 days
- integrity evidence artifacts: 180 days

## Canonical Contract

- `configs/security/data-protection.yaml`
- `configs/observability/retention.yaml`
