# Full Stack Local

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `operations`

## What

Single-command path for local stack bring-up and smoke validation.

## Why

Avoids drift across runbooks and keeps local verification reproducible.

## Contracts

- Interface uses make targets only.
- Canonical sequence is fixed and validated by docs lint.
- Extended meta target: `ops-full`.

## Failure modes

If one target fails, stop and inspect `artifacts/ops/latest/`.

## How to verify

```bash
$ make ops-up ops-deploy ops-warm ops-smoke
$ make ops-full
```

Expected output: all targets exit zero and smoke queries return valid responses.

Meta target: `make ops-full`.

## See also

- [E2E Index](./e2e/INDEX.md)
- [Load Index](./load/INDEX.md)
- [Observability Index](./observability/INDEX.md)
