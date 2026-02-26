# How To Run Locally

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `operations`

## What

Defines the single canonical local run entrypoint.

## Why

Prevents command drift across docs and local workflows.

## Contracts

- Canonical command is `make root-local`.
- Local docs must not prescribe alternate full-run entrypoints.

## Failure modes

Using non-canonical run paths causes inconsistent local validation.

## How to verify

```bash
make root-local
```

Expected output: lane summary is printed with pass/fail status and artifact paths.
