---
title: Extend Perf Benchmarks Through Dev Atlas
audience: contributor
type: how-to
stability: stable
owner: bijux-atlas-performance
last_reviewed: 2026-03-05
tags:
  - operations
  - perf
---

# Extend Perf Benchmarks Through Dev Atlas

Perf benchmark orchestration lives in `bijux-dev-atlas`, not under `ops/` scripts.

## Add a new benchmark

1. Add the benchmark command implementation in `crates/bijux-dev-atlas/src/commands/perf.rs`.
2. Add contract and golden tests in `crates/bijux-dev-atlas/tests/`.
3. Add fixtures under `ops/cli/perf/fixtures/` if static inputs are required.
4. Run `bijux-dev-atlas perf cli-ux bench --format json` and persist generated artifacts.

## Benchmark execution commands

- `bijux-dev-atlas perf cli-ux bench`
- `bijux-dev-atlas perf cli-ux diff <baseline> <candidate>`

These commands replace legacy script-based benchmark runners.
