---
title: Client Test Troubleshooting
audience: developer
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - client
  - testing
  - troubleshooting
---

# Client test troubleshooting

## Missing lockfile

Symptom: `missing deterministic lockfile ... requirements.lock`.

Resolution:

1. Create or refresh `crates/bijux-atlas-client-python/requirements.lock`.
2. Re-run: `bijux-dev-atlas clients python test --client atlas-client --install-deps`.

## Integration tests skipped

Symptom: integration tests show skipped status.

Resolution:

1. Set `ATLAS_CLIENT_ALLOW_INTEGRATION=1`.
2. Re-run `clients python test`.

## Performance tests skipped

Symptom: performance tests show skipped status.

Resolution:

1. Set `ATLAS_CLIENT_RUN_PERF=1`.
2. Re-run `clients python test`.

## Network-restricted run

Use:

```bash
bijux-dev-atlas clients python test --client atlas-client --skip-network --format json
```

This mode is intended for deterministic CI lanes where outbound network is not permitted.
