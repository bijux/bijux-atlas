---
title: Atlas Client Usage Examples
audience: developer
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - client
  - examples
---

# Atlas client usage examples

Canonical usage examples for the Python SDK live in:

- `crates/bijux-atlas-python/examples/`
- `crates/bijux-atlas-python/examples/usage/`
- `crates/bijux-atlas-python/examples/INDEX.md`
- `crates/bijux-atlas-python/examples/MATURITY.md`

Verification entrypoint:

```bash
cargo run -p bijux-dev-atlas -- clients examples-verify --client atlas-client --format json
cargo run -p bijux-dev-atlas -- clients examples-run --client atlas-client --format json
```
