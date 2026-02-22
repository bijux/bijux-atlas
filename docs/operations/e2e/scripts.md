# E2E Interface

- Owner: `bijux-atlas-operations`

## What

Defines the supported E2E interface through `atlasctl ops e2e run --suite ...`.

## Why

Prevents direct script coupling and keeps operational entrypoints stable.

## Contracts

- Canonical runner: `./bin/atlasctl ops e2e run --suite smoke|k8s-suite|realdata`
- Modes: `--fast`, `--no-deploy`, `--profile <id>`
- Suite SSOT: `ops/e2e/suites/suites.json`

## Failure modes

Direct script usage in docs or workflows causes unsupported entrypoint drift.

## How to verify

```bash
$ ./bin/atlasctl ops e2e run --suite smoke
$ ./bin/atlasctl ops e2e run --suite k8s-suite --profile kind
```

Expected output: targets exit zero and produce artifacts under `artifacts/ops/`.

## See also

- [E2E Index](INDEX.md)
- [Full Stack Locally](../full-stack-local.md)
- [Makefile Surface](../../development/makefiles/surface.md)
