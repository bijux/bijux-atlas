---
title: Control-plane
audience: contributor
type: concept
stability: stable
owner: platform
last_reviewed: 2026-03-01
tags:
  - control-plane
  - automation
related:
  - docs/development/index.md
  - docs/operations/index.md
---

# Control-plane

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: provide the canonical control-plane product surface for contributors and CI operators.

## Why this section exists

The control-plane replaces ad-hoc scripts with explicit, contract-governed commands, reports, and gates.
In this documentation set, the `Control plane` is the `bijux dev atlas` surface that enforces those rules consistently.

## Curated guide map

- [Why a control-plane](why-a-control-plane.md)
- [How suites work](how-suites-work.md)
- [Static and effect mode](static-and-effect-mode.md)
- [Capabilities model](capabilities-model.md)
- [Reports schema](reports-schema.md)
- [CI report consumption](ci-report-consumption.md)
- [Extend the control-plane](extend-control-plane.md)
- [Debug failing checks](debug-failing-checks.md)
- [CLI reference](cli-reference.md)

## Entry Points

- Start here for contributor automation and check orchestration.
- Use [Development](../development/index.md) for workflow context.

## Stable entrypoints

- `cargo run -q -p bijux-dev-atlas -- --help`
- `cargo run -q -p bijux-dev-atlas -- check --help`
- `cargo run -q -p bijux-dev-atlas -- docs --help`
- `make ci-pr`
- `make docs-build`

## Verify success

A contributor can discover command surfaces, reproduce CI checks locally, and understand output contracts without reading governance internals.

## Next steps

- [Development](../development/index.md)
- Glossary
- [What is Bijux Atlas](../product/what-is-bijux-atlas.md)
