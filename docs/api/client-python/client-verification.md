---
title: Client Verification
audience: developer
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - client
  - verification
---

# Client verification

Run full Python SDK verification:

```bash
cargo run -p bijux-dev-atlas -- clients verify --client atlas-client --format json
```

Verification scope:

- documentation generation drift
- docs schema and compatibility matrix coverage
- examples policy compliance
- offline-friendly test posture guarantees

Evidence artifacts:

- `artifacts/clients/atlas-client/examples-run-evidence.json`
- `artifacts/clients/atlas-client/verify-evidence.json`
- `artifacts/clients/atlas-client/verify-evidence.md`
- `artifacts/clients/atlas-client/python-test-evidence.json`

Related guides:

- [Client Test Policy](./client-test-policy.md)
- [Client Test Troubleshooting](./client-test-troubleshooting.md)
