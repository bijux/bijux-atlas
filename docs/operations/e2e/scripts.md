# E2E Interface

- Owner: `bijux-atlas-operations`

## What

Defines the supported E2E interface through `make ops-*` targets.

## Why

Prevents direct script coupling and keeps operational entrypoints stable.

## Contracts

- Stack lifecycle: `ops-up`, `ops-down`, `ops-reset`
- Dataset flow: `ops-publish-medium`, `ops-deploy`, `ops-warm`
- Runtime checks: `ops-smoke`, `ops-metrics-check`, `ops-traces-check`
- Suites: `ops-k8s-tests`, `ops-load-smoke`, `ops-load-full`, `ops-realdata`

## Failure modes

Direct script usage in docs or workflows causes unsupported entrypoint drift.

## How to verify

```bash
$ make ops-up ops-deploy ops-warm ops-smoke
$ make ops-k8s-tests
```

Expected output: targets exit zero and produce artifacts under `artifacts/ops/`.

## See also

- [E2E Index](INDEX.md)
- [Full Stack Locally](../full-stack-local.md)
- [Makefile Surface](../../development/makefiles/surface.md)
