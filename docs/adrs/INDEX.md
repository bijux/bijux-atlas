# ADR Index

- Owner: `docs-governance`

## What

Index of architecture decision records.

## Why

ADRs capture significant technical decisions and tradeoffs.

## Scope

Applies to decisions stored under `docs/adrs/`.

## Non-goals

Does not duplicate full decision content from each ADR.

## Contracts

- [ADR-0001 Workspace Boundaries](ADR-0001-workspace-boundaries-and-effects.md)
- [ADR-0002 SQLite Serving Store](ADR-0002-sqlite-serving-store.md)
- [ADR-0003 Federated Registry Merge](ADR-0003-federated-registry-deterministic-merge.md)
- [ADR-0004 Plugin Dispatch](ADR-0004-plugin-contract-and-umbrella-dispatch.md)
- [ADR-0005 Security Defaults](ADR-0005-security-defaults-and-enterprise-controls.md)

## Failure modes

Missing ADR linkage causes decision context loss.

## How to verify

```bash
$ make docs
```

Expected output: ADR links resolve and docs checks pass.

## See also

- [Architecture Index](../architecture/index.md)
- [Contracts Index](../contracts/contracts-index.md)
- [Terms Glossary](../_style/terms-glossary.md)
