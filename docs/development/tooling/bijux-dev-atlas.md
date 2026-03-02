---
title: bijux-dev-atlas
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-02
tags:
  - development
  - tooling
  - control-plane
related:
  - docs/development/tooling/index.md
  - docs/control-plane/cli-reference.md
---

# `bijux-dev-atlas`

`bijux-dev-atlas` is the repository control-plane binary. It owns contract execution, docs validation, config validation, and CI-facing check orchestration.

## Primary command groups

- `check`: discover, explain, and run registered checks and suites.
- `docs`: build, validate, and regenerate docs reference surfaces.
- `configs`: inspect and validate configuration schemas and contracts.
- `contracts`: run domain-specific contract suites and emit reports.
- `ci`: run CI-oriented validation entrypoints locally.

## Typical local commands

```bash
cargo run -q -p bijux-dev-atlas -- check list
cargo run -q -p bijux-dev-atlas -- docs validate --format json
cargo run -q -p bijux-dev-atlas -- configs verify --format json
```

## Capability gates

- `--allow-write` is required for commands that regenerate or rewrite artifacts.
- `--allow-subprocess` is required for commands that shell out to external tooling.
- `--allow-network` is required for commands that intentionally touch remote systems.

## Next

- [CLI reference](../../control-plane/cli-reference.md)
- [Make wrapper surface](make.md)
- [How To Change Docs](../how-to-change-docs.md)
