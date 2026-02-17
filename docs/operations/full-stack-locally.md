# Full Stack Locally

- Owner: `operations`

## What

Single-command path for local stack bring-up and smoke validation.

## Why

Avoids drift across runbooks and keeps local verification reproducible.

## Contracts

- Interface uses make targets only.
- Canonical sequence is fixed and validated by docs lint.

## Failure modes

If one target fails, stop and inspect `artifacts/ops/latest/`.

## How to verify

```bash
$ make ops-up ops-deploy ops-warm ops-smoke
```

Expected output: all targets exit zero and smoke queries return valid responses.

## See also

- [E2E Index](./e2e/INDEX.md)
- [Load Index](./load/INDEX.md)
- [Observability Index](./observability/INDEX.md)
