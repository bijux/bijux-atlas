# No Direct Path Usage

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines the policy for command execution in operations workflows.

## Why

Prevents filesystem-path drift and keeps interface stable through make targets.

## Contracts

- Use `make` targets only in runbooks and operator docs.
- Do not run `ops manifests and control-plane commands` directly in documented workflows.
- Canonical workflows are `ops-*` targets from `make help` (for example `ops-full-pr`).

## Failure modes

Direct script/path usage causes drift when internals move.

## How to verify

```bash
$ make no-direct-scripts
$ make check-gates
```

Expected output: both targets pass.

## See also

- [Operations Index](INDEX.md)
- [Full Stack Locally](full-stack-local.md)
- [Makefile Surface](../development/makefiles/surface.md)
