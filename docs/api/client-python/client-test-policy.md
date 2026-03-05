---
title: Client Test Policy
audience: developer
type: policy
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - client
  - testing
---

# Client test policy

Default client verification is delegated to `bijux-dev-atlas`.

## Default lanes

- `clients verify`: docs, examples, schema, compatibility checks.
- `clients python test`: python product tests from the client crate.

## Test scope tags

Each python test file must declare one `# test_scope:` tag:

- `unit`
- `integration`
- `perf`

## Runtime guards

- Integration tests require `ATLAS_CLIENT_ALLOW_INTEGRATION=1`.
- Performance tests require `ATLAS_CLIENT_RUN_PERF=1`.
- `clients python test --skip-network` enforces offline execution mode.

## Deterministic dependencies

`clients python test` requires `packages/bijux-atlas-python/requirements.lock`.

## Evidence

- `artifacts/clients/atlas-client/python-test-evidence.json`
- `artifacts/clients/atlas-client/verify-evidence.json`
- `artifacts/clients/atlas-client/verify-evidence.md`
