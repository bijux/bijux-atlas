# Repository Overview

- Owner: `docs-governance`
- Stability: `stable`

## What

One-page system graph for repo responsibilities.

## Why

Provides a fast map of where product, contracts, runtime, and operations live.

## Scope

Top-level repo modules and their main dependency flow.

## Non-goals

Does not replace crate-level API docs.

## Contracts

- `docs/contracts/` is SSOT for machine-facing contracts.
- `crates/` implements runtime behavior.
- `ops/` contains local stack, deploy, load, and observability workflows.
- `makefiles/` is the only task orchestration surface.

## Failure modes

- Missing overview causes onboarding drift and wrong entrypoints.

## How to verify

```bash
$ make docs
```

Expected output: links and section contracts pass.

## See also

- [Architecture Diagram](system-graph.md)
- [Repository Layout](../development/repo-layout.md)
- [Terms Glossary](../_style/terms-glossary.md)
