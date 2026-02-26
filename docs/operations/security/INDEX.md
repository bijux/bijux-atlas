# Operations Security Index

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `mixed`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `docs-governance`

## What

Index page for `operations/security` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
$ make ops-check
```

Expected output: docs build and ops contract checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
- `ops-check`
- `ops-ci`
